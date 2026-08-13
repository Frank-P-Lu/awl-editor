use super::*;

const OUTLINE_TOP_LEVEL_MAX: u8 = 2;

const OUTLINE_GAP_ROWS: Rows = Rows(0.5);

fn is_top_level(level: u8) -> bool {
    level <= OUTLINE_TOP_LEVEL_MAX
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::render) enum OutlineRung {
    Faint,
    Content,
}

impl OutlineRung {
    fn color(self) -> glyphon::Color {
        let ink = match self {
            OutlineRung::Faint => theme::faint(),
            OutlineRung::Content => theme::base_content(),
        };
        ink.to_glyphon()
    }
}

const OUTLINE_EDGE_FADE_ALPHA: f32 = 0.45;

fn faded(color: glyphon::Color, f: f32) -> glyphon::Color {
    let a = (color.a() as f32 * f).round().clamp(0.0, 255.0) as u8;
    glyphon::Color::rgba(color.r(), color.g(), color.b(), a)
}

/// The ANCESTOR CHAIN of the heading at `idx` — the nearest preceding heading at
/// each strictly-shallower level, walking UP the document-order list. A heading at
/// level L's ancestors are the nearest preceding headings whose level is `< L`, one
/// per distinct shallower level actually crossed: walk backward tracking the deepest
/// level still "needed", pushing (and shrinking the need to) each strictly-shallower
/// heading, until nothing shallower than H1 remains. An H1 — or any heading with no
/// shallower heading before it — has an EMPTY chain. Returned nearest-first; order
/// does not matter to callers (they test membership).
///
/// Worked example (the spec's own): `H1, H2, H2, H3(idx 3)` → the chain of the H3 is
/// `{2, 0}` — the nearest preceding H2 (idx 2) and the H1 (idx 0), never the earlier
/// sibling H2 (idx 1).
pub(in crate::render) fn ancestor_chain(
    headings: &[crate::markdown::Heading],
    idx: usize,
) -> Vec<usize> {
    let mut out = Vec::new();
    if idx >= headings.len() {
        return out;
    }
    let mut need = headings[idx].level;
    for i in (0..idx).rev() {
        if need <= 1 {
            break; // nothing is shallower than H1
        }
        let lvl = headings[i].level;
        if lvl < need {
            out.push(i);
            need = lvl;
        }
    }
    out
}

pub(in crate::render) fn row_rung(is_current: bool) -> OutlineRung {
    if is_current {
        OutlineRung::Content
    } else {
        OutlineRung::Faint
    }
}

fn first_top_level(headings: &[crate::markdown::Heading]) -> Option<usize> {
    headings.iter().position(|h| is_top_level(h.level))
}

fn group_gap_before(
    headings: &[crate::markdown::Heading],
    first_top: Option<usize>,
    i: usize,
) -> bool {
    is_top_level(headings[i].level) && first_top.is_some_and(|f| i > f)
}

const OUTLINE_REVEAL_DEPTH: bool = false;

fn reveal_depth_on() -> bool {
    OUTLINE_REVEAL_DEPTH || std::env::var_os("AWL_OUTLINE_REVEAL").is_some()
}

fn reveal_shown_with(
    headings: &[crate::markdown::Heading],
    current: Option<usize>,
    reveal: bool,
) -> Vec<usize> {
    if !reveal {
        return (0..headings.len()).collect();
    }
    let section_of = |i: usize| (0..=i).rev().find(|&j| is_top_level(headings[j].level));
    let cur_section = current.and_then(&section_of);
    (0..headings.len())
        .filter(|&i| is_top_level(headings[i].level) || section_of(i) == cur_section)
        .collect()
}

fn reveal_shown(headings: &[crate::markdown::Heading], current: Option<usize>) -> Vec<usize> {
    reveal_shown_with(headings, current, reveal_depth_on())
}

/// ANCHOR-TO-COLUMN: the outline block's LEFT origin (device px). The block HUGS the
/// writing column — its RIGHT edge sits at `right_edge` (`column_left − gap`), so the
/// left origin is `right_edge − block_w` (the block's own natural shaped width),
/// clamped never to cross left of the `min_left` margin pad. Lines stay INTERNALLY
/// left-aligned from this origin (the level indentation still reads left-to-right);
/// only the whole block's x moves, so it tracks the column as the page resizes. Pure
/// (unit-testable without a GPU): the `block_w > right_edge − min_left` overflow is
/// the graceful-hide case, handled earlier by the char floor, and the clamp is the
/// belt-and-braces floor.
fn outline_block_left(right_edge: f32, block_w: f32, min_left: f32) -> f32 {
    crate::render::plan::plan_outline_left(right_edge, block_w, min_left)
}

fn outline_collapsed_marker(hidden: usize) -> String {
    format!(" ({hidden})")
}

/// One decided OUTLINE ROW for a frame: the label (ALREADY fit to one line through
/// [`rowlayout::fit_primary`], with the [`outline_collapsed_marker`] suffix already
/// folded in when `collapsed`), its composite ink `rung` ([`row_rung`]), whether it
/// is the `current` heading (for the sidecar/tests — the ink already encodes the lit
/// path), whether a half-row group `gap_before` renders above it (already
/// window-adjusted: never on the first visible row), and the source heading's 0-based
/// document `line` — the CLICK-TO-JUMP target ([`TextPipeline::outline_hit_line`] maps
/// a pointer y to the row and jumps the caret there), so the click reuses the outline's
/// OWN row geometry rather than a parallel hit-test.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::render) struct OutlineRow {
    pub(in crate::render) label: String,
    pub(in crate::render) rung: OutlineRung,
    pub(in crate::render) faded: bool,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::render) current: bool,
    pub(in crate::render) gap_before: bool,
    pub(in crate::render) line: usize,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::render) collapsed: bool,
}

struct OutlineLayout {
    right_edge: f32,
    avail: f32,
    top: f32,
    lines: Vec<OutlineRow>,
}

struct OutlineBand {
    left: f32,
    width: f32,
    y_top: f32,
    label: String,
}

impl TextPipeline {
    /// The persistent margin OUTLINE's fully decided layout for this frame, or
    /// `None` when the outline is HIDDEN outright — the graceful-hide rule, ANY of:
    /// the feature is OFF ([`crate::outline::outline_on`]); NOT page mode (no margin
    /// to hold it — edge-to-edge stays clean); a non-markdown buffer or a
    /// heading-free document (`!md_enabled` / `outline_headings.is_empty()`); the
    /// margin is too narrow for even a stub title ([`rowlayout::OUTLINE_MIN_CHARS`],
    /// so a narrow window collapses the outline exactly as it collapses the gutter);
    /// or there is no vertical room for even one row above the gutter's reserved
    /// bottom band. Otherwise the visible lines are each fit to ONE line through the
    /// shared elision door, each carrying its composite ink rung + group-gap flag.
    ///
    /// **Long-doc FOLLOW (the chosen default):** when there are more headings than
    /// the margin height holds, the visible window SLIDES to keep the CURRENT
    /// heading on screen — the same [`super::scroll_window`] the pickers use, with
    /// the current heading as the "selection". The row budget is shrunk until the
    /// windowed rows PLUS their internal group gaps fit the vertical band (gaps eat
    /// half a row each). So the section you are reading never scrolls out of the
    /// margin; short documents show every heading from the top.
    fn outline_layout(&self, height: u32) -> Option<OutlineLayout> {
        if !crate::outline::outline_on() || !crate::page::page_on() {
            return None;
        }
        // SUMMONED OVERLAYS OWN THE MARGINS: while ANY overlay is open — every
        // `overlay_active` card, whatever backdrop treatment it asks for (that set is
        // `OverlayKind::keeps_backdrop_crisp`'s, not this owner's) — the persistent
        // margin outline yields, returning on dismissal. Consistent with the lava
        // rail-carve precedent: chrome cedes the margin to the summoned surface. This
        // is the ONE outline-layout owner every reader (draw, hit-test, frost pills,
        // stars, sidecar `outline_visible`) routes through: the whole outline darkens.
        if self.overlay_active {
            return None;
        }
        if !self.md_enabled || self.outline_headings.is_empty() {
            return None;
        }
        let label = crate::markdown::type_scale::LABEL;
        let left_pad = self.edge_pad();
        let gap = self.metrics.char_width * MARGIN_COLUMN_GAP_CHARS.0;
        let right_edge = self.column_left() - gap;
        let avail = right_edge - left_pad;
        if avail <= 0.0 {
            return None;
        }
        let label_char_w = self.metrics.char_width * label;
        let avail_chars = if label_char_w > 0.0 {
            (avail / label_char_w).floor().max(0.0) as usize
        } else {
            0
        };
        if avail_chars < rowlayout::OUTLINE_MIN_CHARS {
            return None;
        }
        let row_h = self.metrics.line_height * label;
        let top = self.text_origin_top();
        let gutter_reserve = row_h * 3.0 + self.metrics.px_physical(readout::CANVAS_INSET);
        let avail_h = height as f32 - gutter_reserve - top;
        let max_rows = if row_h > 0.0 {
            (avail_h / row_h).floor().max(0.0) as usize
        } else {
            0
        };
        if max_rows == 0 {
            return None;
        }
        let full = &self.outline_headings;
        let current = self.outline_current(); // index into the FULL list

        let shown = reveal_shown(full, current);
        if shown.is_empty() {
            return None;
        }
        let len = shown.len();
        let sel = current
            .and_then(|c| shown.iter().position(|&i| i == c))
            .unwrap_or(0);

        let first_top = first_top_level(full);
        let gap_full: Vec<bool> = shown
            .iter()
            .map(|&i| group_gap_before(full, first_top, i))
            .collect();

        let mut budget = max_rows;
        let (win_top, count) = loop {
            let (wt, cnt) = super::scroll_window(len, sel, 0, budget);
            let gaps = (wt + 1..wt + cnt).filter(|&j| gap_full[j]).count();
            let used = cnt as f32 * row_h + gaps as f32 * row_h * OUTLINE_GAP_ROWS.0;
            if used <= avail_h || cnt <= 1 || budget <= 1 {
                break (wt, cnt);
            }
            budget -= 1;
        };

        let clips_above = win_top > 0;
        let clips_below = win_top + count < len;
        let last_vis = count.saturating_sub(1);
        let lines = (win_top..win_top + count)
            .enumerate()
            .map(|(vis, pos)| {
                let idx = shown[pos]; // index into the FULL heading list
                let h = &full[idx];
                let label = rowlayout::fit_primary_end(&h.label(), avail_chars);
                let is_current = current == Some(idx);
                let rung = row_rung(is_current);
                let clipped_edge = (vis == 0 && clips_above) || (vis == last_vis && clips_below);
                let faded = clipped_edge && !is_current;
                let gap_before = vis > 0 && gap_full[pos];
                let hidden = self
                    .fold_tails
                    .iter()
                    .find(|t| t.line == h.line)
                    .map(|t| t.hidden);
                let collapsed = hidden.is_some();
                let label = match hidden {
                    Some(n) => format!("{label}{}", outline_collapsed_marker(n)),
                    None => label,
                };
                OutlineRow {
                    label,
                    rung,
                    faded,
                    current: is_current,
                    gap_before,
                    line: h.line,
                    collapsed,
                }
            })
            .collect();
        Some(OutlineLayout {
            right_edge,
            avail,
            top,
            lines,
        })
    }

    pub(in crate::render) fn outline_visible(&self, height: u32) -> bool {
        self.outline_layout(height).is_some()
    }

    /// The persistent outline's interactive margin band. Toasts treat this as
    /// active chrome for the same reason pointer hit-testing does: covering the
    /// words would also cover the only click target that jumps to that heading.
    /// This deliberately comes from `outline_layout`, so graceful hiding and
    /// summoned-overlay ownership cannot diverge between paint and avoidance.
    pub(in crate::render) fn outline_keepout_rect(&self, height: u32) -> Option<[f32; 4]> {
        let layout = self.outline_layout(height)?;
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        if row_h <= 0.0 {
            return None;
        }
        let slots = crate::render::plan::plan_outline_slots(
            layout.top,
            row_h,
            OUTLINE_GAP_ROWS.0,
            layout.lines.iter().map(|row| (row.line, row.gap_before)),
        );
        let first = slots.first()?;
        let last = slots.last()?;
        let left = self.edge_pad();
        Some([
            left,
            first.y,
            (layout.right_edge - left).max(0.0),
            (last.y + row_h - first.y).max(0.0),
        ])
    }

    pub(in crate::render) fn lava_frost_pill_rects(&mut self, height: u32) -> Vec<[f32; 4]> {
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        if row_h <= 0.0 {
            return Vec::new();
        }
        let pad_x =
            crate::lava::frost_px(crate::lava::FROST_PILL_PAD_X, self.metrics.zoom, self.dpi);
        let inset_y = row_h * crate::lava::FROST_PILL_INSET_Y_FRAC;
        let mut rects = Vec::new();
        for band in self.outline_ink_bands(height) {
            if band.width > 0.0 {
                rects.push([
                    (band.left - pad_x).max(0.0),
                    band.y_top + inset_y,
                    band.left + band.width + pad_x,
                    band.y_top + row_h - inset_y,
                ]);
            }
            if rects.len() >= crate::lava::MAX_FROST_PILLS {
                break;
            }
        }
        rects
    }

    fn outline_ink_bands(&mut self, height: u32) -> Vec<OutlineBand> {
        let Some(mut layout) = self.outline_layout(height) else {
            return Vec::new();
        };
        self.outline_pixel_fit(&mut layout);
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        if row_h <= 0.0 {
            return Vec::new();
        }
        let widths: Vec<f32> = layout
            .lines
            .iter()
            .map(|r| self.measure_outline_label_px(&r.label))
            .collect();
        let block_w = widths.iter().copied().fold(0.0_f32, f32::max);
        let left = outline_block_left(layout.right_edge, block_w, self.edge_pad());
        let slots = crate::render::plan::plan_outline_slots(
            layout.top,
            row_h,
            OUTLINE_GAP_ROWS.0,
            layout.lines.iter().map(|row| (row.line, row.gap_before)),
        );
        let mut bands = Vec::with_capacity(layout.lines.len());
        for (i, row) in layout.lines.iter().enumerate() {
            bands.push(OutlineBand {
                left,
                width: widths[i],
                y_top: slots[i].y,
                label: row.label.clone(),
            });
        }
        bands
    }

    pub(in crate::render) fn outline_frost_seeds(&mut self, height: u32) -> Vec<[f32; 4]> {
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        if row_h <= 0.0 {
            return Vec::new();
        }
        let feather = crate::lava::FROST_FEATHER_PX;
        let r_row = crate::render::frost_seed_radius(row_h, feather, self.metrics.zoom, self.dpi);
        let skirt = crate::lava::frost_px(feather, self.metrics.zoom, self.dpi);
        let pad_x =
            crate::lava::frost_px(crate::lava::FROST_PILL_PAD_X, self.metrics.zoom, self.dpi);
        let yc_off = row_h * 0.5;
        let mut seeds = Vec::new();
        for band in self.outline_ink_bands(height) {
            if band.width <= 0.0 {
                continue;
            }
            crate::render::push_text_seeds(
                &mut seeds,
                band.left - pad_x,
                band.width + 2.0 * pad_x,
                band.y_top + yc_off,
                r_row,
                skirt,
                &band.label,
            );
            if seeds.len() >= crate::lava::MAX_FROST_SEEDS {
                seeds.truncate(crate::lava::MAX_FROST_SEEDS);
                break;
            }
        }
        seeds
    }

    /// THE FROST SEED-CACHE KEY ROWS for the outline — a CHEAP char-level snapshot
    /// (each fitted `label` + its group-gap / current flags + source `line`) of the
    /// drawn outline this frame, WITHOUT the pixel-fit shaping. Read only by
    /// [`TextPipeline::frost_seed_key`] to detect when the seeded margin text
    /// changed (a follow-window slide, an edited heading, a reveal shift): the
    /// pixel-fit that the real seeds ride only trims further with zoom/DPI/face,
    /// which the key hashes separately, so this char-level view is a faithful cache
    /// discriminator with no per-frame shaping cost.
    pub(in crate::render) fn outline_key_rows(
        &self,
        height: u32,
    ) -> Option<Vec<(String, bool, bool, usize)>> {
        let layout = self.outline_layout(height)?;
        Some(
            layout
                .lines
                .iter()
                .map(|r| (r.label.clone(), r.gap_before, r.current, r.line))
                .collect(),
        )
    }

    /// PERSISTENT MARGIN OUTLINE: the CURRENT heading's [`ancestor_chain`] — the
    /// indices (into [`Self::outline_headings`]) of the headings the caret is nested
    /// inside, EMPTY when the caret sits above the first heading or the current
    /// heading is top-level. A pure function of the heading list + [`Self::outline_current`].
    /// Reported in the capture sidecar's `outline` block (a STRUCTURAL fact — the
    /// caret's heading nesting — so a headless test can assert it deterministically
    /// without GPU; the render no longer LIGHTS ancestors — only the current row is
    /// `Content` — but the nesting is still worth reporting).
    pub fn outline_ancestors(&self) -> Vec<usize> {
        match self.outline_current() {
            Some(c) => ancestor_chain(&self.outline_headings, c),
            None => Vec::new(),
        }
    }

    /// CLICK-TO-JUMP: the 0-based document LINE of the outline row under the pointer
    /// at `(px, py)` (physical px), or `None` when the pointer is off the outline (the
    /// outline is hidden, or the point lands outside the block's x band / between
    /// rows). Reuses the outline's OWN row geometry — the SAME [`Self::outline_layout`]
    /// the pixels ride (its follow slice, group gaps, and `top`/`row_h`), never a
    /// parallel hit-test — so a click can never target a row the frame didn't draw.
    /// The x band is the whole left margin `[TEXT_LEFT, right_edge]` (the block hugs
    /// `right_edge`); each row occupies its own `row_h`, with a half-row `gap_before`
    /// added ABOVE a group-opening row (matching the render's vertical stacking).
    ///
    /// A benign, user-approved navigation affordance (DESIGN.md outline amendment:
    /// "click-to-jump only") — NOT a resizable/focusable sidebar. The live App wires
    /// it in `app/input/mouse.rs` (`outline_click`) and lights the pointing-hand cursor over
    /// a row (`cursor_shape`), both gated on the outline actually being drawn.
    pub fn outline_hit_line(&self, px: f32, py: f32, height: u32) -> Option<usize> {
        let layout = self.outline_layout(height)?;
        if px < self.edge_pad() || px > layout.right_edge {
            return None;
        }
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        if row_h <= 0.0 {
            return None;
        }
        let slots = crate::render::plan::plan_outline_slots(
            layout.top,
            row_h,
            OUTLINE_GAP_ROWS.0,
            layout.lines.iter().map(|row| (row.line, row.gap_before)),
        );
        crate::render::plan::hit_outline_slot(
            &slots,
            px,
            py,
            [self.edge_pad(), layout.right_edge],
            row_h,
        )
    }

    pub(in crate::render) fn prepare_outline(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let faint = theme::faint().to_glyphon();
        // Scale BOTH font size and line height to LABEL so the rows nest tightly
        // (this buffer is standalone, not row-aligned to the doc — like the gutter).
        self.outline_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(m.font_size * label, m.line_height * label),
        );
        // NEVER wrap: a heading row is meant to be exactly one visual line (the
        // fixed row_h `outline_hit_line` advances by leans on it) — matches the
        // sibling chrome buffers' explicit pattern (`preview.rs`, `readout.rs`,
        // the theme picker, the overlay panel). Without this, a label whose
        // shaped pixel width slips past the char-count estimate below would
        // WORD-WRAP onto a second visual row, pushing every row under it one
        // row_h too low and desyncing the draw from the hit-test geometry.
        self.outline_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let base = panel_attrs();
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let Some(mut layout) = self.outline_layout(height) else {
            self.outline_buffer
                .set_size(&mut self.font_system, Some(1.0), Some(m.line_height));
            self.outline_buffer.set_text(
                &mut self.font_system,
                "",
                &base.clone().color(faint),
                Shaping::Advanced,
                None,
            );
            self.outline_buffer
                .shape_until_scroll(&mut self.font_system, false);
            let area = TextArea {
                buffer: &self.outline_buffer,
                left: 0.0,
                top: -1000.0,
                scale: 1.0,
                bounds,
                default_color: faint,
                custom_glyphs: &[],
            };
            self.outline_renderer
                .prepare(
                    device,
                    queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    [area],
                    &mut self.swash_cache,
                )
                .map_err(|e| anyhow::anyhow!("glyphon outline prepare failed: {e:?}"))?;
            return Ok(());
        };
        // `outline_layout`'s label is only a CHAR-COUNT ESTIMATE (a monospace mean
        // `char_width`) — correct it to the MEASURED shaped pixel width so a label
        // carrying disproportionately wide glyphs never spills past `avail` (see
        // `outline_pixel_fit`'s own doc comment). After this, every row genuinely
        // fits its one line — `Wrap::None` above never actually needs to clip.
        self.outline_pixel_fit(&mut layout);
        // Each visible heading is now fit to one line BY MEASURED PIXELS (through
        // `outline_pixel_fit` → the shared `rowlayout::fit_primary_end_to_px` door),
        // so this box NEVER lays raw, possibly-overflowing text into its wrap width.
        // Build the visual lines:
        // each heading row coloured by its rung, preceded by a HALF-ROW blank gap line
        // where `gap_before` (a lone space carrying half-height metrics, so cosmic-text
        // — which keys each row's height off its glyphs' line heights — collapses that
        // line to a half-row breath while the label rows stay full LABEL height).
        let row_h = m.line_height * label;
        let gap_metrics = GlyphMetrics::new(m.font_size * label, row_h * OUTLINE_GAP_ROWS.0);
        let mut vlines: Vec<(String, glyphon::Color, bool)> = Vec::new();
        for row in &layout.lines {
            if row.gap_before {
                vlines.push((" ".to_string(), faint, true));
            }
            let color = if row.faded {
                faded(row.rung.color(), OUTLINE_EDGE_FADE_ALPHA)
            } else {
                row.rung.color()
            };
            vlines.push((row.label.clone(), color, false));
        }
        let n_rows = layout.lines.len();
        let gap_count = layout.lines.iter().filter(|r| r.gap_before).count();
        let pieces: Vec<(String, glyphon::Color, bool)> = vlines
            .into_iter()
            .enumerate()
            .map(|(i, (text, color, gap))| {
                let joined = if i == 0 { text } else { format!("\n{text}") };
                (joined, color, gap)
            })
            .collect();
        let spans: Vec<(&str, Attrs)> = pieces
            .iter()
            .map(|(t, c, gap)| {
                let mut attrs = base.clone().color(*c);
                if *gap {
                    attrs = attrs.metrics(gap_metrics);
                }
                (t.as_str(), attrs)
            })
            .collect();
        let total_h = n_rows as f32 * row_h + gap_count as f32 * row_h * OUTLINE_GAP_ROWS.0 + 1.0;
        self.outline_buffer
            .set_size(&mut self.font_system, Some(layout.avail), Some(total_h));
        let default_attrs = base.clone().color(faint);
        self.outline_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.outline_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut block_w = 0.0_f32;
        for run in self.outline_buffer.layout_runs() {
            block_w = block_w.max(run.line_w);
        }
        let left = outline_block_left(layout.right_edge, block_w, self.edge_pad());
        let area = TextArea {
            buffer: &self.outline_buffer,
            left,
            top: layout.top,
            scale: 1.0,
            bounds,
            default_color: faint,
            custom_glyphs: &[],
        };
        self.outline_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon outline prepare failed: {e:?}"))?;
        Ok(())
    }

    /// PIXEL-FIT CORRECTION — the fix for the click-jumps-to-the-wrong-heading bug:
    /// [`Self::outline_layout`]'s `row.label` is only a CHAR-COUNT ESTIMATE (fit
    /// against a monospace MEAN `char_width`), which under-predicts a title carrying
    /// disproportionately WIDE glyphs (an emoji/symbol-heavy heading — the repro is
    /// a run of `⌘`). Such a label can shape wider than `layout.avail` even though
    /// it fit the char budget, and — before `Wrap::None` was added to
    /// [`Self::prepare_outline`] — would WORD-WRAP onto a second visual line,
    /// pushing every row below it one `row_h` lower than [`Self::outline_hit_line`]
    /// (which advances by a FIXED `row_h` per row) assumes: a click below the
    /// wrapped row would land one heading early. This corrects each row's label to
    /// its MEASURED shaped pixel width via [`rowlayout::fit_primary_end_to_px`], so
    /// one heading is genuinely one visual row BY CONSTRUCTION — `Wrap::None` then
    /// never actually needs to clip anything.
    ///
    /// Self-contained (sets its own metrics/wrap on [`Self::outline_buffer`]) so it
    /// behaves identically whether called from the real draw path
    /// ([`Self::prepare_outline`], which already set them moments earlier — a
    /// harmless redundant call) or straight from a test
    /// ([`Self::outline_draw_report`], which never touches the buffer otherwise).
    fn outline_pixel_fit(&mut self, layout: &mut OutlineLayout) {
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        self.outline_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(m.font_size * label, m.line_height * label),
        );
        self.outline_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let avail = layout.avail;
        for row in &mut layout.lines {
            row.label = rowlayout::fit_primary_end_to_px(&row.label, avail, |s| {
                self.measure_outline_label_px(s)
            });
        }
    }

    /// MEASURE the shaped pixel width of `label`, at whatever metrics
    /// [`Self::outline_buffer`] currently carries (the outline's LABEL scale), via a
    /// throwaway single-line shape into that SAME buffer — mirroring
    /// `measure_spell_content_w`'s reuse-then-redraw pattern (`chrome/overlay.rs`):
    /// harmless because the buffer is always fully re-shaped with the real composite
    /// text before it ever reaches the GPU (either by [`Self::prepare_outline`]'s own
    /// later `set_rich_text`, or — in the hidden/test-only case — simply never drawn).
    pub(in crate::render) fn measure_outline_label_px(&mut self, label: &str) -> f32 {
        self.outline_buffer
            .set_size(&mut self.font_system, None, None);
        self.outline_buffer.set_text(
            &mut self.font_system,
            label,
            &panel_attrs(),
            Shaping::Advanced,
            None,
        );
        self.outline_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut w = 0.0f32;
        for run in self.outline_buffer.layout_runs() {
            w = w.max(run.line_w);
        }
        w
    }

    #[cfg(test)]
    pub(in crate::render) fn outline_draw_report(
        &mut self,
        height: u32,
    ) -> Option<Vec<OutlineRow>> {
        let mut layout = self.outline_layout(height)?;
        self.outline_pixel_fit(&mut layout);
        Some(layout.lines)
    }

    #[cfg(test)]
    pub(in crate::render) fn outline_avail_px(&self, height: u32) -> Option<f32> {
        self.outline_layout(height).map(|l| l.avail)
    }

    #[cfg(test)]
    pub(in crate::render) fn outline_top_px(&self, height: u32) -> Option<f32> {
        self.outline_layout(height).map(|l| l.top)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::Heading;

    fn h(level: u8, text: &str) -> Heading {
        Heading {
            level,
            text: text.into(),
            line: 0,
        }
    }

    #[test]
    fn ancestor_chain_is_the_nearest_shallower_heading_per_level() {
        let hs = [h(1, "T"), h(2, "A"), h(2, "B"), h(3, "Deep")];
        let mut anc = ancestor_chain(&hs, 3);
        anc.sort_unstable();
        assert_eq!(
            anc,
            vec![0, 2],
            "H3's ancestors = nearest preceding H2 (idx2) + the H1 (idx0)"
        );

        assert_eq!(
            ancestor_chain(&hs, 0),
            Vec::<usize>::new(),
            "an H1 has no ancestors"
        );

        assert_eq!(
            ancestor_chain(&hs, 2),
            vec![0],
            "an H2's ancestor is the H1, never a sibling H2"
        );

        let deep = [h(1, "1"), h(2, "2"), h(3, "3"), h(4, "4")];
        assert_eq!(
            ancestor_chain(&deep, 3),
            vec![2, 1, 0],
            "a deep H4 lifts H3,H2,H1 nearest-first"
        );

        let jump = [h(3, "deep first"), h(2, "later")];
        assert_eq!(
            ancestor_chain(&jump, 0),
            Vec::<usize>::new(),
            "the first heading never has an ancestor"
        );
        assert_eq!(
            ancestor_chain(&jump, 1),
            Vec::<usize>::new(),
            "an H2 with only a deeper H3 before it has none"
        );
    }

    #[test]
    fn row_rung_is_two_state_current_content_else_faint() {
        assert_eq!(
            row_rung(true),
            OutlineRung::Content,
            "the current heading is Content (dark)"
        );
        assert_eq!(
            row_rung(false),
            OutlineRung::Faint,
            "every other heading is Faint"
        );
        assert_ne!(
            row_rung(true),
            row_rung(false),
            "the current row reads above the rest"
        );
    }

    #[test]
    fn outline_block_left_hugs_the_column_right_edge() {
        let right_edge = 300.0;
        let min_left = 16.0;
        let block_w = 120.0;
        let left = outline_block_left(right_edge, block_w, min_left);
        assert!(
            (left + block_w - right_edge).abs() < 1e-3,
            "the block's right edge hugs the column"
        );
        assert!(left >= min_left);
        let fat = right_edge - min_left + 50.0;
        assert_eq!(
            outline_block_left(right_edge, fat, min_left),
            min_left,
            "clamps at the margin pad"
        );
    }

    #[test]
    fn faded_scales_only_the_alpha_channel() {
        let c = glyphon::Color::rgba(120, 130, 140, 200);
        assert_eq!(faded(c, 1.0), c, "f=1 is a no-op");
        let half = faded(c, 0.5);
        assert_eq!(
            (half.r(), half.g(), half.b()),
            (120, 130, 140),
            "RGB unchanged"
        );
        assert_eq!(half.a(), 100, "alpha halved: round(200 * 0.5)");
        assert_eq!(faded(c, 0.0).a(), 0, "f=0 is fully transparent");
    }

    #[test]
    fn reveal_shown_gates_deep_headings_to_the_caret_section() {
        let hs = [
            h(1, "T"),
            h(2, "A"),
            h(3, "a1"),
            h(3, "a2"),
            h(2, "B"),
            h(3, "b1"),
        ];
        assert_eq!(
            reveal_shown_with(&hs, Some(2), false),
            vec![0, 1, 2, 3, 4, 5],
            "reveal off shows every heading"
        );
        assert_eq!(
            reveal_shown_with(&hs, Some(2), true),
            vec![0, 1, 2, 3, 4],
            "in section A: A's H3s show, B's H3 is hidden, all top-level shown"
        );
        assert_eq!(
            reveal_shown_with(&hs, Some(4), true),
            vec![0, 1, 4, 5],
            "in section B: B's H3 shows, A's H3s hidden"
        );
        assert_eq!(
            reveal_shown_with(&hs, None, true),
            vec![0, 1, 4],
            "above the first heading, no section is current, so only H1/H2 show"
        );
    }

    #[test]
    fn group_gap_precedes_each_non_first_top_level_section() {
        let hs = [
            h(1, "Title"),
            h(2, "At a glance"),
            h(3, "detail"),
            h(2, "Each world"),
            h(3, "Mopoke"),
            h(2, "The fonts"),
        ];
        let ft = first_top_level(&hs);
        assert_eq!(ft, Some(0), "the H1 title is the first top-level section");
        let gaps: Vec<bool> = (0..hs.len())
            .map(|i| group_gap_before(&hs, ft, i))
            .collect();
        assert_eq!(
            gaps,
            vec![false, true, false, true, false, true],
            "no gap before the title; a gap before each later H2; never before an H3"
        );

        let one = [h(1, "Only"), h(3, "sub"), h(3, "sub2")];
        let ft1 = first_top_level(&one);
        let gaps1: Vec<bool> = (0..one.len())
            .map(|i| group_gap_before(&one, ft1, i))
            .collect();
        assert_eq!(
            gaps1,
            vec![false, false, false],
            "a single top-level section has no gaps"
        );

        let no_h1 = [h(2, "A"), h(3, "a"), h(2, "B")];
        let ftn = first_top_level(&no_h1);
        assert_eq!(ftn, Some(0));
        let gapsn: Vec<bool> = (0..no_h1.len())
            .map(|i| group_gap_before(&no_h1, ftn, i))
            .collect();
        assert_eq!(
            gapsn,
            vec![false, false, true],
            "the first H2 opens no group; the second does"
        );
    }
}
