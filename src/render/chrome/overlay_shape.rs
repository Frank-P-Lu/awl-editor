use super::overlay_timeline::right_bind_lines;
use super::*;

mod names;

// `right_bind_lines` preserves the secondary buffer's vertical contract:
// its first label leads with exactly `header_rows` blank lines, so
// `secondary_top() + (header_rows + row) * lh == row_top(row)`.
// A contextual card has zero header lines, and must therefore gain none.
// The retired `.max(1)` looked defensive but shifted every secondary label
// and its selected ink onto the following row for that one card family.
// Timeline fitting now shares this same constructor from `overlay_timeline`:
// elision may change a label's width, never its display-line identity.
// Keeping the constructor named at this import documents why the general
// overlay shaper and the timeline-specific pixel refinement cannot each grow
// their own nearly-identical newline arithmetic again.

/// PHYSICAL, and the placard family is chrome's one honest exception.
///
/// The wordmark is FRAME, not text: its size comes from the window's own short
/// side and is deliberately independent of zoom (`render::tests::
/// overlay_personality::placard_size_is_window_scaled_not_zoom_scaled` fails by
/// name if that changes). Its inset from the canvas corner is part of that same
/// frame, in the same device space as the canvas it insets from — enrolling it
/// in `zoom * dpi` would make the poster's margin chase the text zoom while the
/// poster itself held still.
const PLACARD_INSET: Physical = Physical(12.0);

/// PHYSICAL, and the only honest reading: it is the byte-stable capture
/// canvas's own device height, the fixed point the placard ladder is anchored
/// at. Scaling a reference would move the fixed point it exists to hold still.
const PLACARD_REFERENCE_SHORT_SIDE: Physical = Physical(crate::capture::CANVAS_HEIGHT as f32);

/// PROPORTIONAL PLACARD SIZING — the FROZEN calibration anchor: the value of the
/// markdown TITLE rung at the moment the placard fractions were calibrated by eye
/// (pre-Ladder-J `type_scale::TITLE`, 1.8). Deliberately a LITERAL, decoupled from
/// the live document ladder: the per-world placard look is a user-picked identity,
/// and a later document-ladder retune (Ladder J moved TITLE to 1.6) must never
/// silently resize every world's wordmark. Chrome reads THIS anchor; only the
/// document reads the live rung.
const PLACARD_CALIBRATION_TITLE: f32 = 1.8;

/// A RATIO (placard height per unit of window short side), not a length.
const PLACARD_HEIGHT_PER_SCALE: f32 =
    crate::render::FONT_SIZE * PLACARD_CALIBRATION_TITLE / PLACARD_REFERENCE_SHORT_SIDE.0;

/// PHYSICAL for [`PLACARD_INSET`]'s reason: a floor on a size that is already
/// derived from the device canvas, in that same space.
const PLACARD_MIN_HEIGHT: Physical = Physical(56.0);

/// PHYSICAL: this ceiling exists to bound the GLYPH ATLAS, and atlas capacity
/// is a device resource measured in device pixels. Doubling it on a 2x panel
/// would double the rasterized mask this bound was chosen to cap.
const PLACARD_MAX_HEIGHT: Physical = Physical(512.0);

/// PLACARD ATLAS-SAFETY (AtlasFull fix, 2026-07-17) — the geometric step the
/// wordmark's font size is SNAPPED to. The proportional sizing above tracks the
/// window short side CONTINUOUSLY, so a live resize sweep asked the shaper for a
/// fresh giant Archivo-Black size every pixel of drag travel; each distinct size
/// rasterizes its own glyph set into the ONE shared glyph atlas
/// (`TextPipeline::atlas` — document body text, rows and placard all live there),
/// and a fast sweep on a large display filled it faster than the per-frame
/// `atlas.trim()` reclaimed → [`glyphon::PrepareError::AtlasFull`], which blanked
/// the card AND starved the document text sharing the atlas. Snapping to a ~3%
/// ladder bounds the whole clamp band [`PLACARD_MIN_HEIGHT`]..[`PLACARD_MAX_HEIGHT`]
/// to ≈ `log(512/56)/log(1.03) ≈ 75` distinct rungs — a wordmark this large moves
/// a pixel or two between rungs, imperceptible, while the atlas stays bounded no
/// matter how long the drag. 3–4% was the board's call; 1.03 sits at the calm end.
const PLACARD_SIZE_STEP: f32 = 1.03;

/// Snap a placard font target (px) to the geometric ladder of
/// [`PLACARD_SIZE_STEP`], anchored at `anchor` (the world's REFERENCE-canvas
/// size — so the 1200×800 reference short side is an EXACT fixed point and every
/// default-zoom placard capture stays byte-identical). `round_down` FLOORS to the
/// ladder (used by the shrink-to-fit path, so the snapped size never exceeds the
/// fit target and the wordmark still fits the canvas); otherwise rounds to the
/// nearest rung. Because BOTH the main and shrink paths anchor at the same
/// `anchor`, every size either path can produce is `anchor · step^k` for integer
/// `k` — ONE ladder, so the two paths' union stays bounded (never a product). Pure
/// → the bounded-ladder law is unit-testable without a GPU (see
/// `render::tests::overlay_personality`).
pub(in crate::render) fn snap_placard_size(target: f32, anchor: f32, round_down: bool) -> f32 {
    if !matches!(target.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater))
        || !matches!(anchor.partial_cmp(&0.0), Some(std::cmp::Ordering::Greater))
    {
        return target;
    }
    let steps = (target / anchor).ln() / PLACARD_SIZE_STEP.ln();
    let k = if round_down {
        steps.floor()
    } else {
        steps.round()
    };
    anchor * (k * PLACARD_SIZE_STEP.ln()).exp()
}

const STIPPLE_COVERAGE_THRESHOLD: u8 = 0x80;

fn placard_origin(
    corner: theme::PlacardCorner,
    anchor: (f32, f32, f32, f32),
    w: f32,
    h: f32,
    inset: f32,
) -> (f32, f32) {
    let (ax, ay, aw, ah) = anchor;
    let x = match corner {
        theme::PlacardCorner::TL | theme::PlacardCorner::BL | theme::PlacardCorner::Auto => {
            (ax + inset).min((ax + aw - w).max(ax))
        }
        theme::PlacardCorner::TR | theme::PlacardCorner::BR => (ax + aw - inset - w).max(ax),
    };
    let y = match corner {
        theme::PlacardCorner::TL | theme::PlacardCorner::TR => {
            (ay + inset).min((ay + ah - h).max(ay))
        }
        theme::PlacardCorner::BL | theme::PlacardCorner::BR | theme::PlacardCorner::Auto => {
            (ay + ah - inset - h).max(ay)
        }
    };
    (x, y)
}

fn widest_run(buffer: &GlyphBuffer) -> f32 {
    let mut w = 0.0f32;
    for run in buffer.layout_runs() {
        w = w.max(run.line_w);
    }
    w
}

/// The `Bars` INLINE-SHORTCUT arm's shaped strings: each planned row's elided
/// primary label, plus its chord appended after a fixed gap.
fn inline_shortcut_rows(
    row_labels: &[String],
    items: &[Option<usize>],
    right_labels: &[String],
    total_chars: usize,
    elide: bool,
) -> (Vec<String>, Vec<String>) {
    let full = rowlayout::full_budget(total_chars);
    let fit = |l: &String| {
        if elide {
            rowlayout::fit_primary(l, full)
        } else {
            l.clone()
        }
    };
    let rows = row_labels.iter().map(fit).collect();
    let tail = |item: &Option<usize>| match item.and_then(|i| right_labels.get(i)) {
        Some(s) if !s.is_empty() => format!("{}{}", super::INLINE_SHORTCUT_GAP, s),
        _ => String::new(),
    };
    (rows, items.iter().map(tail).collect())
}

/// THE QUERY BEAT'S OWN GLYPH-FREE LINE, when the band plans one — one space at
/// the planned beat's height. A LINE, not line height on a line that has glyphs:
/// cosmic-text centres a glyph run in its box, so a beat folded into the query
/// field's line draws the field's text half a beat below its own bar.
fn push_beat_spacer<'a>(
    spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
    attrs: glyphon::Attrs<'a>,
    font_size: f32,
    beat: Option<f32>,
) {
    if let Some(beat) = beat {
        spans.push(("\n", attrs.clone()));
        spans.push((" ", attrs.metrics(GlyphMetrics::new(font_size, beat))));
    }
}

impl TextPipeline {
    /// THE PLACARD RENDERER — the one owner of [`theme::TitleStyle::Placard`].
    /// Shapes the picker's own title text (`overlay_title`, the ONE owner of
    /// the announced text — see `OverlayKind::title`'s doc; already gated
    /// empty for the two kinds that orient via their own modal prompt
    /// instead) as a large, corner-anchored, DIM wordmark into
    /// `placard_buffer` — sized by `scale` over the document body's own font
    /// size × the frozen calibration TITLE anchor
    /// ([`PLACARD_CALIBRATION_TITLE`]), so a world dials how loud its
    /// wordmark reads with ONE number, never a second magic constant — and
    /// CAPPED by the canvas itself (the fit-to-canvas shrink below): the
    /// window's own width is the ceiling the dial can never shout past.
    /// Uppercased (a taste call, flagged — a display wordmark reads as a
    /// title card, not running prose).
    ///
    /// Returns the wordmark's natural `(x, y, w, h)` draw rect, or `None`
    /// when this frame draws no placard: the active [`theme::TitleStyle`]
    /// (probe-forced or the active world's own, see
    /// `render::effective_title_style`) is `InlinePrefix` (every world
    /// today), the picker is the header-less spell popup (no title line at
    /// all — `header_rows == 0`), or the kind draws no title (Rename/
    /// InsertLink — `overlay_title` is already empty for those).
    ///
    /// THE SCREEN-CORNER ANCHOR (settled — supersedes the card-clipped
    /// original): the wordmark anchors to the FULL CANVAS corners and draws
    /// as a dim watermark OVER the scrim, BEHIND the card (the Persona-style
    /// bleed the card-clip original deliberately declined). The caller clips
    /// the upload to the WHOLE CANVAS (not the tighter card rect), and the
    /// wordmark's `TextArea` is still uploaded FIRST in the text batch, so
    /// the rows/query line always composite OVER it — legibility first, and
    /// the dimmed document below still shows through (the wordmark rides the
    /// text pass, above the scrim quad).
    ///
    /// COMPOSES WITH THE FACETED LAYOUT (fixed post-launch — a prior round's
    /// guard also bailed on `geom.theme`, blanking the placard on every
    /// picker [`crate::facets::scheme`] facets — the Cmd-P palette and the
    /// Settings menu included, the two surfaces that matter most): there is
    /// nothing kind-specific about this fn's OWN work — it anchors to the
    /// CANVAS (`self.window_w`/`self.window_h`, identical on both
    /// `overlay_geometry`'s flat branch and `theme_overlay_geometry`'s
    /// faceted branch) and reads only `geom.header_rows` +
    /// `self.overlay_title`/`self.placard_buffer`. The faceted shaper
    /// (`theme_picker.rs::overlay_shape_theme`) fills the SAME
    /// `panel_buffer` the flat shaper does, and both are uploaded through the
    /// SAME `overlay_upload_text` (`overlay.rs`) which always pushes the
    /// placard's `TextArea` FIRST (drawn behind) — so a faceted card's lens
    /// strip + section-grouped rows composite OVER the wordmark exactly like
    /// a flat card's query line + rows do, no new wiring needed. This
    /// includes the LITERAL Theme kind itself: nothing in `theme_picker.rs`
    /// depends on the card being placard-free (no state it reads or writes
    /// changes), so excluding it once the mechanism composes for free would
    /// just be an inconsistent special case — the exact smell
    /// `CLAUDE.md`'s "merge, don't align" principle warns against.
    pub(in crate::render) fn overlay_shape_placard(
        &mut self,
        geom: &OverlayGeom,
    ) -> Option<(f32, f32, f32, f32)> {
        if geom.header_rows == 0 || self.overlay_title.is_empty() {
            return None;
        }
        let (corner, scale, ink) = match crate::render::effective_title_style() {
            theme::TitleStyle::Placard { corner, scale, ink } => (corner, scale, ink),
            theme::TitleStyle::InlinePrefix => return None,
        };
        if geom.card_narrow {
            return None;
        }
        let corner = crate::render::derived_placard_corner(
            corner,
            crate::render::resolve_overlay_anchor(self.overlay_align),
        );
        let short_side = self.window_w.min(self.window_h);
        let reference_size = scale
            * PLACARD_HEIGHT_PER_SCALE
            * self.metrics.px_physical(PLACARD_REFERENCE_SHORT_SIDE);
        // ATLAS-SAFETY: snap the continuous window-tracked size to the ladder BEFORE the
        // clamp, so a live resize sweep produces a BOUNDED set of distinct giant sizes
        // (never a fresh atlas entry per drag pixel — the AtlasFull fix).
        let font_size = snap_placard_size(
            scale * PLACARD_HEIGHT_PER_SCALE * short_side,
            reference_size,
            false,
        )
        .clamp(
            self.metrics.px_physical(PLACARD_MIN_HEIGHT),
            self.metrics.px_physical(PLACARD_MAX_HEIGHT),
        );
        let mut line_height = font_size * 1.1;
        let metrics = GlyphMetrics::new(font_size, line_height);
        self.placard_buffer
            .set_metrics(&mut self.font_system, metrics);
        self.placard_buffer
            .set_size(&mut self.font_system, None, None);
        self.placard_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let text = self.overlay_title.to_uppercase();
        let color = theme::placard_ink(ink).to_glyphon();
        self.placard_buffer.set_text(
            &mut self.font_system,
            &text,
            &chrome_attrs().color(color),
            Shaping::Advanced,
            None,
        );
        self.placard_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut w = widest_run(&self.placard_buffer);
        if w <= 0.0 {
            return None;
        }
        let reserve = self.menubar_reserve();
        let anchor = (0.0, reserve, self.window_w, self.window_h - reserve);
        // FIT THE CANVAS (the minimum-window overflow fix — found live by the
        // standing-policy audit): `scale` is a per-world LOUDNESS dial, not a
        // fit guarantee — a long title ("version history") at the app's own
        // enforced minimum window shapes ~2.6x wider than the whole canvas
        // and hard-clipped off the right edge. When the natural width exceeds
        // the anchor minus BOTH insets, shrink the font size proportionally
        // and re-lay out: cosmic-text shapes normalized (per-em) advances and
        // multiplies by the buffer metrics' font size at LAYOUT time, so ONE
        // linear re-metric lands the width at the target (residual float
        // noise is absorbed by `placard_origin`'s clamps). A comfortable
        // window never enters this branch — byte-identical. An ADAPTIVE
        // policy with no config knob, the `adaptive_column_left` idiom; the
        // stipple rasterizer reads the same re-shaped buffer, so it fits for
        // free.
        let inset = self.metrics.px_physical(PLACARD_INSET);
        let avail = anchor.2 - 2.0 * inset;
        if avail > 0.0 && w > avail {
            // ATLAS-SAFETY: snap the fit target DOWN to the same ladder the main size
            // rode. Flooring guarantees the shrunk mark still fits `avail` (the snapped
            // size never exceeds `font_size · avail/w`), and anchoring at the same
            // `reference_size` keeps every width-only-resize shrink size on the ONE
            // bounded ladder — so a horizontal drag of a long title can't fill the atlas
            // either (the width-sweep the main clamp alone never covered).
            let shrunk = snap_placard_size(font_size * (avail / w), reference_size, true);
            line_height = shrunk * 1.1;
            self.placard_buffer.set_metrics(
                &mut self.font_system,
                GlyphMetrics::new(shrunk, line_height),
            );
            self.placard_buffer
                .shape_until_scroll(&mut self.font_system, false);
            w = widest_run(&self.placard_buffer);
        }
        let (x, y) = placard_origin(corner, anchor, w, line_height, inset);
        Some((x, y, w, line_height))
    }

    /// THE STIPPLE PLACARD's rasterizer: the coverage RUNS of the just-shaped
    /// `placard_buffer`'s glyphs, as 1px-tall rects positioned at the
    /// wordmark's draw origin — fed to the `placard_stipple` pipeline, whose
    /// dither branch then keeps only the Bayer-selected pixels (the SAME
    /// matrix + shader branch as the Wagtail highlight stipple — one pattern
    /// language, per the round's rule). CPU-rasterized off the SAME swash
    /// cache glyphon itself uses (the morph caret's established idiom —
    /// `render/caret.rs`'s mask rasterization), so the letterforms are the
    /// real shaped glyphs, deterministic across captures (no clock, no
    /// random: coverage is pure shaping, the Bayer cut is pure position).
    /// Emitting RUNS (not per-pixel rects) keeps the instance count at
    /// O(rows × glyphs), not O(pixels). Color-glyph (emoji) images are
    /// skipped — a wordmark title has none, and a coverage mask is the only
    /// content the stipple contract can honor.
    pub(in crate::render) fn placard_stipple_rects(&mut self, origin: (f32, f32)) -> Vec<[f32; 4]> {
        let (px, py) = origin;
        let mut glyphs: Vec<(CacheKey, f32, f32)> = Vec::new();
        for run in self.placard_buffer.layout_runs() {
            let baseline_y = py + run.line_y;
            for g in run.glyphs.iter() {
                glyphs.push((g.physical((0.0, 0.0), 1.0).cache_key, px + g.x, baseline_y));
            }
        }
        let Self {
            swash_cache,
            font_system,
            ..
        } = self;
        let mut rects: Vec<[f32; 4]> = Vec::new();
        for (key, pen_x, baseline_y) in glyphs {
            let Some(img) = swash_cache.get_image(font_system, key).as_ref() else {
                continue;
            };
            if img.placement.width == 0
                || img.placement.height == 0
                || img.content != SwashContent::Mask
            {
                continue;
            }
            let gw = img.placement.width as usize;
            let x0 = pen_x + img.placement.left as f32;
            let y0 = baseline_y - img.placement.top as f32;
            for (row, cols) in img.data.chunks_exact(gw).enumerate() {
                let y = y0 + row as f32;
                let mut start: Option<usize> = None;
                for (col, &alpha) in cols.iter().enumerate() {
                    match (alpha >= STIPPLE_COVERAGE_THRESHOLD, start) {
                        (true, None) => start = Some(col),
                        (false, Some(s)) => {
                            rects.push([x0 + s as f32, y, (col - s) as f32, 1.0]);
                            start = None;
                        }
                        _ => {}
                    }
                }
                if let Some(s) = start {
                    rects.push([x0 + s as f32, y, (gw - s) as f32, 1.0]);
                }
            }
        }
        rects
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::render) fn overlay_shape_text(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
        elide: bool,
    ) -> bool {
        self.overlay_right_shown = false;
        if geom.theme {
            return self.shape_faceted(geom, plan, ink, muted, selected_ink, vis, elide);
        }
        // The shaped line `k` and the PLANNED row `k` are one object: a
        // row's item comes off the plan, never a second `top_idx + k` computed here.
        let items: Vec<Option<usize>> = plan.rows().iter().map(|r| r.item).collect();
        let label = |i: &Option<usize>| i.and_then(|i| self.overlay_items.get(i)).cloned();
        let row_labels: Vec<String> = items
            .iter()
            .map(label)
            .map(Option::unwrap_or_default)
            .collect();

        let right_labels = self.overlay_right_labels();
        let has_right = self.spell_has_secondary(right_labels);
        let chord = |i: &Option<usize>| {
            i.and_then(|i| right_labels.get(i))
                .map_or("", |s| s.as_str())
        };
        let bind_strs = right_bind_lines(plan.header_rows(), items.iter().map(chord));

        // One shared row budget: card text width against the widest right label. `Split`/
        // `Full` elide the names to their granted budget (the historical math);
        // `Measure` shapes them UNELIDED and lets the shaped pixels decide below.
        //
        // WILD-MENU SLANT PROBE (env-gated; `slant_tax == 0.0` on every normal
        // run — byte-identical): the deepest row's stair offset is subtracted
        // from the effective row span BEFORE the rowlayout math, so elision
        // respects the reduced width (a shifted row can never paint past the
        // card's right text edge). Rows still flow through `rowlayout` — the
        // law is untouched; only the width it budgets against shrinks.
        let slant = crate::render::overlay_slant();
        let slant_tax = slant
            .map(|s| crate::render::slant_max_offset(&s, plan.candidate_rows()))
            .unwrap_or(0.0);
        let slant_text_w = (geom.text_w - slant_tax).max(0.0).min(
            self.diagonal_cluster_budget(geom, plan.rows().len())
                .unwrap_or(f32::INFINITY),
        );
        let char_w = self.overlay_char_width();
        let total_chars = if char_w > 0.0 {
            (slant_text_w / char_w).floor() as usize
        } else {
            usize::MAX
        };
        if has_right && super::bars_inline_shortcut() {
            let (rows, trailing) =
                inline_shortcut_rows(&row_labels, &items, right_labels, total_chars, elide);
            self.shape_overlay_names(geom, plan, ink, muted, selected_ink, vis, &rows, &trailing);
            return false;
        }
        let widest_right = if has_right {
            Some(
                right_labels
                    .iter()
                    .map(|s| s.chars().count())
                    .max()
                    .unwrap_or(0),
            )
        } else {
            None
        };
        let measured_spell_fits = self.measured_spell_primary_fits(slant_text_w);
        let budget = if !elide || measured_spell_fits {
            None
        } else {
            match rowlayout::plan(total_chars, widest_right) {
                rowlayout::Plan::Full { primary } | rowlayout::Plan::Split { primary } => {
                    Some(primary)
                }
                rowlayout::Plan::Measure => None,
            }
        };
        let rows: Vec<String> = row_labels
            .iter()
            .map(|label| match budget {
                Some(b) => rowlayout::fit_primary(label, b),
                None => label.clone(),
            })
            .collect();
        self.shape_overlay_names(geom, plan, ink, muted, selected_ink, vis, &rows, &[]);
        if !has_right {
            return false;
        }
        self.shape_overlay_right(geom, ink, muted, vis, &bind_strs);

        let name_px = self.widest_candidate_px(geom, plan);
        let right_px = self.widest_right_px();
        let gap_px = rowlayout::GAP_CHARS as f32 * char_w;
        if rowlayout::fits(slant_text_w, gap_px, name_px, right_px) {
            self.overlay_right_shown = true;
            return true;
        }
        if !elide {
            return false;
        }
        let full = rowlayout::full_budget(total_chars);
        let rows: Vec<String> = row_labels
            .iter()
            .map(|label| rowlayout::fit_primary(label, full))
            .collect();
        self.shape_overlay_names(geom, plan, ink, muted, selected_ink, vis, &rows, &[]);
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_faceted(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
        elide: bool,
    ) -> bool {
        // Diagonal rows occupy one side of their spine rather than the whole
        // card. Shape against that real side territory before deciding whether
        // a secondary cell fits, so narrow cards elide/yield instead of drawing
        // a measured cluster through the spine or off its planned span.
        let mut shaped_geom = geom.for_rows();
        if let Some(budget) = self.diagonal_cluster_budget(geom, plan.rows().len()) {
            shaped_geom.text_w = shaped_geom.text_w.min(budget);
        }
        let right_labels = self.overlay_right_labels();
        let has_right = !right_labels.is_empty();
        // Timeline metadata remains a quiet, right-aligned lane even in a Bars
        // world whose ordinary shortcut chords hug the primary label inline.
        let hug_inline = has_right && super::bars_inline_shortcut() && !geom.workspace;
        let trailing: Vec<String> = if hug_inline {
            geom.plan
                .iter()
                .map(|line| match line {
                    PlanLine::Item(i) => match right_labels.get(*i) {
                        Some(s) if !s.is_empty() => {
                            format!("{}{}", super::INLINE_SHORTCUT_GAP, s)
                        }
                        _ => String::new(),
                    },
                    PlanLine::Location(_) | PlanLine::Header(_) => String::new(),
                })
                .collect()
        } else {
            Vec::new()
        };
        let bind_strs: Vec<String> = if has_right && !hug_inline {
            right_bind_lines(
                plan.header_rows(),
                geom.plan.iter().map(|line| match line {
                    PlanLine::Item(i) => right_labels.get(*i).map(|s| s.as_str()).unwrap_or(""),
                    PlanLine::Location(_) | PlanLine::Header(_) => "",
                }),
            )
        } else {
            Vec::new()
        };
        self.overlay_shape_theme(
            &shaped_geom,
            plan,
            ink,
            muted,
            selected_ink,
            vis,
            &trailing,
            elide,
        );
        // `rowlayout` begins from a character-grid estimate, while the diagonal
        // side territory is a hard pixel bound. A display face can be wider than
        // that estimate (especially at 2×), so re-shape once against the actual
        // widest label ratio before seating its fixed accessory column.
        if let Some(budget) = self.diagonal_cluster_budget(geom, plan.rows().len()) {
            let measured = self.widest_candidate_px(&shaped_geom, plan);
            if measured > budget && measured > 0.0 {
                shaped_geom.text_w *= budget / measured;
                self.overlay_shape_theme(
                    &shaped_geom,
                    plan,
                    ink,
                    muted,
                    selected_ink,
                    vis,
                    &trailing,
                    elide,
                );
            }
        }
        if !has_right || hug_inline {
            return false;
        }
        if let Some(shown) =
            self.shape_timeline_right_to_fit(&shaped_geom, plan, ink, muted, vis, elide)
        {
            return shown;
        }
        self.shape_overlay_right(&shaped_geom, ink, muted, vis, &bind_strs);

        // THE NO-OVERLAP LAW, extended to the faceted path: unlike the
        // flat shaper, `shape_theme_spans`'s primary NEVER reserves budget for a
        // secondary at all (its char estimate is always `full_budget`), so a
        // genuinely long chord/time/git label beside a full-width primary had NO
        // yield mechanism — the two could sit closer than the flat path's own
        // `rowlayout::fits` bar allows, or a proportional-font primary the
        // estimate under-measured could shade into the secondary's column. Read
        // the SAME real shaped pixels the flat path's law reads; the secondary
        // YIELDS (whole column dropped, `false`) rather than ever painting
        // toward a name — the primary stays exactly as already shaped (it never
        // budgeted FOR a secondary, so it needs no re-shape on yield).
        let name_px = self.widest_candidate_px(&shaped_geom, plan);
        let right_px = self.widest_right_px();
        let slant = crate::render::overlay_slant();
        let slant_tax = slant
            .map(|s| crate::render::slant_max_offset(&s, plan.candidate_rows()))
            .unwrap_or(0.0);
        let slant_text_w = (shaped_geom.text_w - slant_tax).max(0.0);
        let gap_px = rowlayout::GAP_CHARS as f32 * self.overlay_char_width();
        if !rowlayout::fits(slant_text_w, gap_px, name_px, right_px) {
            return false;
        }
        self.overlay_right_shown = true;
        true
    }

    pub(super) fn overlay_title_prefix(&self, geom: &OverlayGeom) -> String {
        let placard_drawn = matches!(
            crate::render::effective_title_style(),
            theme::TitleStyle::Placard { .. }
        ) && !geom.card_narrow;
        if self.overlay_title.is_empty() || placard_drawn {
            String::new()
        } else {
            format!("{} › ", self.overlay_title)
        }
    }

    pub(super) fn push_overlay_hint_spans<'a>(
        &self,
        spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
        hint: &'a str,
        muted: glyphon::Color,
        gap_rows: usize,
        content_before: bool,
    ) {
        let name_fs = self.overlay_metrics().font_size;
        let hint_fs = name_fs * crate::markdown::type_scale::LABEL;
        let hint_h = self.overlay_hint_h();
        let base = panel_attrs();
        let hk_hint = |c| {
            base.clone()
                .color(c)
                .metrics(GlyphMetrics::new(hint_fs, hint_h))
        };
        let sym_hint = |c| {
            Attrs::new()
                .family(Family::Name(SYMBOL_FAMILY))
                .color(c)
                .metrics(GlyphMetrics::new(hint_fs, hint_h))
        };
        // The blank separator row `overlay_hint_gap_rows` reserves — the
        // row-count owner every geometry family budgets this against, so the
        // reserved row and this drawn one can't drift apart. Its own
        // (`overlay_hint_gap_h`, smaller still than the hint's own row) height:
        // a glyph-free line still needs a real glyph to carry custom metrics
        // (`push_beat_spacer`'s own trick for the query beat), so this is a
        // single invisible space, not a bare second newline.
        if gap_rows > 0 {
            if content_before {
                spans.push(("\n", base.clone().color(muted)));
            }
            spans.push((
                " ",
                base.clone()
                    .color(muted)
                    .metrics(GlyphMetrics::new(hint_fs, self.overlay_hint_gap_h())),
            ));
        }
        if gap_rows > 0 || content_before {
            spans.push(("\n", base.clone().color(muted)));
        }
        push_symbol_split(spans, hint, || hk_hint(muted), || sym_hint(muted));
    }

    #[allow(clippy::too_many_arguments)]
    fn shape_overlay_names(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        ink: glyphon::Color,
        muted: glyphon::Color,
        selected_ink: Option<glyphon::Color>,
        vis: &VisualSelection,
        rows: &[String],
        trailing: &[String],
    ) {
        let fitted_hint = self.overlay_fitted_hint(geom);
        let has_query = geom.header_rows > 0;
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let mut spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        let title_prefix = self.overlay_title_prefix(geom);
        let sigil = "› ";
        let name_fs = self.overlay_metrics().font_size;
        // The field's own PLANNED box height, read rather than re-summed.
        let header_lh = plan
            .query_band()
            .map_or_else(|| self.overlay_lh(), |field| field.height);
        let hk = |c| {
            if geom.header_gap > 0.0 {
                mk(c).metrics(GlyphMetrics::new(name_fs, header_lh))
            } else {
                mk(c)
            }
        };
        let hkc = |c| {
            let a = chrome_attrs().color(c);
            if geom.header_gap > 0.0 {
                a.metrics(GlyphMetrics::new(name_fs, header_lh))
            } else {
                a
            }
        };
        if has_query {
            if title_prefix.is_empty() {
                spans.push((sigil, hk(muted)));
            } else {
                spans.push((title_prefix.as_str(), hkc(muted)));
            }
            spans.push((self.overlay_query.as_str(), hk(ink)));
        }
        push_beat_spacer(&mut spans, mk(muted), name_fs, plan.beat_line());
        self.push_overlay_name_rows(
            &mut spans,
            rows,
            trailing,
            has_query,
            ink,
            muted,
            selected_ink,
            vis,
        );
        if let Some(msg) = &geom.empty {
            if has_query {
                spans.push(("\n", mk(muted)));
            }
            spans.push((msg.as_str(), mk(muted)));
        }
        if geom.hint_rows > 0 {
            self.push_overlay_hint_spans(
                &mut spans,
                fitted_hint.as_str(),
                muted,
                geom.hint_gap_rows,
                has_query || !rows.is_empty() || geom.empty.is_some(),
            );
        }
        let footer_lines: Vec<String> = geom.footer.iter().map(|t| format!("\n{t}")).collect();
        if geom.footer_rows > 0 {
            let faint = theme::faint().to_glyphon();
            let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
            spans.push(("\n", mk(faint))); // the blank separator line
            for line in &footer_lines {
                push_symbol_split(&mut spans, line, || mk(faint), || sym(faint));
            }
        }

        self.panel_buffer
            .set_size(&mut self.font_system, Some(geom.text_w), Some(geom.card_h));
        // Single-line rows: NEVER wrap. A row elided a hair long clips at the card edge
        // instead of spilling onto a second visual row (which overflowed the card).
        self.panel_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let default_attrs = base.clone().color(ink);
        self.panel_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }
    /// The SECONDARY column (shortcut chord / time / git value), one right-aligned
    /// label per display row.
    ///
    /// Its on-band recolor reads the shared [`VisualSelection`], NOT
    /// the logical selected row. Reading the logical row here was the split that
    /// let a pointer move recolor "Switch project…"'s shortcut while the band and
    /// the label it annotates were still on "Go to file…" — two simultaneous
    /// answers to "which command is selected". The secondary now WAITS for the
    /// band, exactly as the primary label already did.
    pub(super) fn shape_overlay_right(
        &mut self,
        geom: &OverlayGeom,
        ink: glyphon::Color,
        muted: glyphon::Color,
        vis: &VisualSelection,
        bind_strs: &[String],
    ) {
        let base = panel_attrs();
        let mono = |c| Attrs::new().family(Family::Monospace).color(c);
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let sel_muted = super::overlay_selected_secondary_ink();
        let mut bind_spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        for (li, s) in bind_strs.iter().enumerate() {
            let c = match sel_muted {
                Some(flip) if vis.reads_selected(li) => flip,
                _ => muted,
            };
            push_symbol_split(&mut bind_spans, s, || mono(c), || sym(c));
        }
        let default_attrs = base.clone().color(ink);
        self.panel_bind_buffer.set_size(
            &mut self.font_system,
            Some(geom.text_w),
            Some(geom.card_h),
        );
        self.panel_bind_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        // ALIGNED TO THE COLUMN'S OWN FLOW, not to the right outright: a mirrored
        // cluster hangs its accessory on the far end and grows it back toward the
        // name, and `overlay_upload_text` seats the buffer through the same flow.
        let align = super::diagonal::accessory_flow(self).align();
        self.panel_bind_buffer.set_rich_text(
            &mut self.font_system,
            bind_spans,
            &default_attrs,
            Shaping::Advanced,
            Some(align),
        );
        self.panel_bind_buffer
            .shape_until_scroll(&mut self.font_system, false);
    }

    fn widest_candidate_px(&self, geom: &OverlayGeom, plan: &OverlayRowPlan) -> f32 {
        let first = geom.shaped_first_row_line();
        let last = first + plan.candidate_rows();
        let mut w = 0.0f32;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i >= first && run.line_i < last {
                w = w.max(run.line_w);
            }
        }
        w
    }

    fn widest_right_px(&self) -> f32 {
        let mut w = 0.0f32;
        for run in self.panel_bind_buffer.layout_runs() {
            w = w.max(run.line_w);
        }
        w
    }

    pub(in crate::render) fn overlay_row_primary_px(
        &self,
        geom: &OverlayGeom,
    ) -> std::collections::BTreeMap<usize, f32> {
        let mut m = std::collections::BTreeMap::new();
        let first = geom.shaped_first_row_line();
        for run in self.panel_buffer.layout_runs() {
            if run.line_i >= first {
                m.insert(run.line_i - first, run.line_w);
            }
        }
        m
    }

    pub(in crate::render) fn overlay_row_secondary_px(
        &self,
        geom: &OverlayGeom,
    ) -> std::collections::BTreeMap<usize, f32> {
        let mut m = std::collections::BTreeMap::new();
        for run in self.panel_bind_buffer.layout_runs() {
            if run.line_i >= geom.header_rows && run.line_w > 0.0 {
                m.insert(run.line_i - geom.header_rows, run.line_w);
            }
        }
        m
    }

    pub(in crate::render) fn overlay_footer_content_px(
        &self,
        geom: &OverlayGeom,
        content_rows: usize,
    ) -> f32 {
        let first = geom.shaped_first_row_line() + content_rows;
        let mut w = 0.0f32;
        for run in self.panel_buffer.layout_runs() {
            if run.line_i >= first {
                w = w.max(run.line_w);
            }
        }
        w
    }
}
