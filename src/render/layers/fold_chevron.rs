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
//! "Rotated labels" section). The mark's SHAPE is not this module's to own: each world
//! draws a real font glyph ([`crate::theme::Theme::fold_mark`] — `›`/`☞`/`▸`, derived
//! from the world's own ornament register, never a hand-picked list), composed into an
//! R8 coverage mask and drawn on a quad rotated onto an axis through
//! [`crate::rotated_label::RotatedLabelPipeline`] — the SAME mechanism the rotated
//! location cue rides (`render/rotated_location.rs`), reused wholesale rather than a
//! second rotation mechanism. What this module owns is the mark's PLACEMENT (the
//! writing column's leading pad), its SUMMONING (caret or hover on a heading), its
//! DIRECTION SOURCE (`folded_headings`), and the shape+size fit that keeps the composed
//! glyph inside that placement box.

use super::*;

/// The gap (chars, at the mark's own scale) between the mark's box and the
/// text it hangs beside.
const GAP_CHARS: f32 = 0.2;
/// The mark's own box width (chars, at the mark's own scale) — sized to hold
/// the WIDEST shipped mark (the Junicode register's manicule) at its own
/// [`crate::theme::FoldMark::size_frac`], not merely a single glyph's advance.
/// `GAP_CHARS + WIDTH_CHARS` must stay comfortably under `3.0` (the fixed
/// `PAGE_TEXT_PAD_CHARS` lead every world reserves): `fold_chevron_scale`'s own
/// pad-fit ratio is `PAGE_TEXT_PAD_CHARS / (GAP_CHARS + WIDTH_CHARS)`, and that
/// ratio must clear the tallest heading rung ([`crate::markdown::heading_scale`]'s
/// own `TITLE` = 1.6) with real margin or H1's mark silently loses its
/// full ladder step to the pad clamp — see
/// `render::tests::fold_chevron_center::fold_chevron_ink_and_box_ride_the_heading_ladder`.
const WIDTH_CHARS: f32 = 1.6;

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

/// This mark's own screen AXIS ANGLE, in [`crate::rotated_label::geometry::
/// label_axis_deg`]'s own protractor convention, from the fold state's turn
/// FRACTION. Every shipped mark is authored pointing RIGHT at rest (the
/// glyph's own natural reading, `›`/`☞`/`▸` all point right unrotated) — `0.0`
/// keeps that reading (collapsed). `label_axis_deg` reads its angle
/// counter-clockwise as a reader would name it, so turning the SAME rightward
/// advance direction to point straight DOWN the screen (expanded) is `-90.0`,
/// which `label_axis_deg`'s own `rem_euclid` folds to its exact `270.0`
/// quadrant branch — never `90.0`, which is the OTHER quarter turn (up) and
/// would mirror the mark rather than rotate it. Proved on rendered pixels, not
/// merely derived: `captures/item-475-glyph-survey`'s
/// `fold_mark_candidates_settle_in_opposite_directions` grades exactly this
/// sign for every candidate before this round picked one.
fn fold_mark_axis_deg(fraction: f32) -> f32 {
    -90.0 * fraction
}

/// Shape one mark glyph into its own one-glyph buffer — the CPU-only half
/// `LabelMask::compose` then rasterises. Mirrors the survey's own
/// `shape_one_char` (`captures/item-475-glyph-survey`'s gallery test): a
/// short-lived buffer just big enough for one glyph, no wrap, no line
/// breaking (a label capability, not the document renderer).
fn shape_fold_mark(
    font_system: &mut FontSystem,
    face: &'static str,
    ch: char,
    px: f32,
) -> GlyphBuffer {
    let mut buf = GlyphBuffer::new(font_system, GlyphMetrics::new(px, px * 1.3));
    buf.set_size(font_system, Some(px * 4.0), Some(px * 2.0));
    buf.set_wrap(font_system, Wrap::None);
    let mut s = String::new();
    s.push(ch);
    buf.set_text(
        font_system,
        &s,
        &Attrs::new().family(Family::Name(face)),
        Shaping::Advanced,
        None,
    );
    buf.shape_until_scroll(font_system, false);
    buf
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

    /// Build + upload this frame's fold-chevron marks — one
    /// [`crate::rotated_label::RotatedLabelPipeline`] per currently-summoned
    /// mark (`fold::chevron_revealed` allows at most two: the caret's own
    /// heading, the hovered heading), grown into [`Self::fold_chevron_labels`]
    /// lazily and parked (`clear()`) past however many marks this frame
    /// actually carries. Every mark this frame draws the SAME world-picked
    /// glyph ([`crate::theme::Theme::fold_mark`]) — the only thing that
    /// differs between two simultaneous marks is their own heading's size
    /// step and fold state, exactly mirroring the old quad implementation's
    /// per-mark independence (`render::tests::fold_chevron_center`'s
    /// mixed-batch law: one mark's pixels are byte-identical to its solo
    /// frame's, regardless of who shares the frame — trivially true here
    /// since each mark owns its own pipeline and texture, not a shared
    /// per-batch uniform).
    ///
    /// The mask CACHE ([`Self::fold_chevron_label_masks`]) is content-addressed
    /// per slot (`LabelMask::matches`), not heading-addressed: two marks
    /// asking for the same glyph at the same size (two same-level headings)
    /// share a cache hit if they land in the same slot, and a slot recomposes
    /// on any real change (fold state, heading level, or the mark swapping
    /// which heading it belongs to) — cheap either way, since at most two
    /// marks ever draw in one frame.
    pub(in crate::render) fn prepare_fold_chevron_marks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let marks = self.fold_chevron_geometries();
        let spec = theme::fold_mark();
        let ink = srgb_u8_to_linear3(theme::fold_afford_chevron_ink().rgba_bytes());

        while self.fold_chevron_labels.len() < marks.len() {
            self.fold_chevron_labels
                .push(crate::rotated_label::RotatedLabelPipeline::new(
                    device,
                    self.format,
                ));
            self.fold_chevron_label_masks.push(None);
        }
        for label in self.fold_chevron_labels.iter_mut().skip(marks.len()) {
            label.clear();
        }

        for (i, g) in marks.iter().enumerate() {
            // The SAME pad-clamped Ladder-J scale the box was built from
            // ([`FoldChevronGeom::scale`]), applied to the heading's own font
            // size — so the mark rides the ladder exactly as the box does —
            // then the register's own [`crate::theme::FoldMark::size_frac`],
            // which exists ONLY because the manicule's ink footprint is wider,
            // at any font size, than the box `WIDTH_CHARS` affords (see that
            // field's own doc); every other register's `1.0` leaves this a
            // pure ladder-scaled font size.
            let font_size = self.metrics.font_size * g.scale * spec.size_frac;
            let buf = shape_fold_mark(&mut self.font_system, spec.face, spec.ch, font_size);
            let stale = match &self.fold_chevron_label_masks[i] {
                Some(mk) => !mk.matches(&buf),
                None => true,
            };
            if stale {
                self.fold_chevron_label_masks[i] = crate::rotated_label::mask::LabelMask::compose(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.swash_cache,
                    &buf,
                );
            }
            let Some(mask) = self.fold_chevron_label_masks[i].as_ref() else {
                // Whitespace-only / no rasterisable ink for this glyph — should
                // not happen for a real mark spec, but leaves nothing to draw
                // rather than upload a stale texture.
                self.fold_chevron_labels[i].clear();
                continue;
            };

            let fraction = self.fold_chevron_turn_fraction(g.line, g.collapsed);
            let axis = crate::rotated_label::geometry::label_axis_deg(fold_mark_axis_deg(fraction));
            // Solve the pen ORIGIN so the mask's own ink CENTER — not its
            // corner — lands on the box's screen center at every turn: the
            // ink center's local offset rotates WITH the axis (the same two
            // basis vectors the shader draws through), so re-deriving it each
            // frame is what keeps the mark visually pivoting in place through
            // the glide rather than orbiting around a fixed corner.
            let ink_box = mask.ink();
            let local_center = [ink_box[0] + ink_box[2] * 0.5, ink_box[1] + ink_box[3] * 0.5];
            let screen_center = [g.left + g.width * 0.5, g.row_center()];
            let offset =
                crate::rotated_label::geometry::label_point([0.0, 0.0], axis, local_center);
            let origin = [screen_center[0] - offset[0], screen_center[1] - offset[1]];

            self.fold_chevron_labels[i].prepare(
                device, queue, width, height, mask, origin, axis, ink, ink, 1.0,
            );
        }
    }
}
