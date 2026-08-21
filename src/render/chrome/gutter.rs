//! PAGE-MODE ORIENTATION GUTTER chrome — the quiet bottom-left stacked label
//! (filename over project), right-aligned to hug the writing column from the
//! margin — widening into the working set's rows when more than one file is open
//! ([`super::gutter_stack`]) — plus its sidecar report. Inherent methods on
//! [`super::TextPipeline`]. The hidden arm and the doc-dimming predicate live in
//! [`super::gutter_hidden`]. See [`super`].

use super::*;

/// The vertical breath (in LABEL rows) added ABOVE the gutter block when carving
/// its local lava corner — a half-row so the feathered top face clears the top
/// glyph. Read by [`TextPipeline::gutter_carve_rect`] and pinned by the
/// gutter-corner bounds law (`theme::tests`).
pub(in crate::render) const GUTTER_CARVE_BREATH: Rows = Rows(0.5);

/// THE PERSISTENT AFFORDANCE'S WORDS — the same two the conflict workspace
/// titles itself with (`OverlayKind::Conflict::title`), so the thing you notice
/// and the place you review it are recognisably one thing. Deliberately NOT the
/// sticky notice's whole sentence: a notice has room to name both resolutions,
/// a margin label has room to name the state.
pub(in crate::render) const GUTTER_CHANGED_LABEL: &str = "changed elsewhere";

/// WHICH of the gutter block's stacked lines a row is. The block is variable —
/// the project line is absent without a project, the affordance is absent
/// without a conflict — so every consumer (the drawn spans, the frost seeds, the
/// carve height, the sidecar, the hit-test) reads the SAME ordered list rather
/// than each re-deriving "is there a second line".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum GutterLine {
    Changed,
    Name,
    /// One member of the WORKING SET, by its slot in the drawn stack. Present
    /// only when the stack replaces [`GutterLine::Name`], so the two are never
    /// both in one block.
    File(usize),
    Project,
}

impl GutterLayout {
    /// The block's lines, TOP to BOTTOM, absent ones omitted. THE one owner of
    /// the block's shape.
    ///
    /// The identity line is EITHER the lone filename or the working set's rows,
    /// never both: with one file open `files` is empty and this returns exactly
    /// the list it always has. Every consumer — the drawn spans, the frost
    /// seeds, the carve height, the hit-test — reads this one list, so widening
    /// the identity moves all four together and none of them re-derives the
    /// block's height from a second count.
    pub(super) fn lines(&self) -> Vec<(&str, GutterLine)> {
        let mut out = Vec::with_capacity(3);
        if !self.changed.is_empty() {
            out.push((self.changed.as_str(), GutterLine::Changed));
        }
        if self.files.is_empty() {
            out.push((self.name.as_str(), GutterLine::Name));
        } else {
            if !self.project.is_empty() {
                out.push((self.project.as_str(), GutterLine::Project));
            }
            for (at, line) in self.files.iter().enumerate() {
                out.push((line.text.as_str(), GutterLine::File(at)));
            }
        }
        if !self.project.is_empty() && self.files.is_empty() {
            out.push((self.project.as_str(), GutterLine::Project));
        }
        out
    }
}

impl TextPipeline {
    /// The page-mode GUTTER's fully decided layout for this frame: the available
    /// RIGHT-aligned box width (px), the filename AND the project line each
    /// ALREADY fit to ONE line independently (never left to cosmic-text's own
    /// word-wrap — see [`Self::prepare_gutter`]'s doc). `None` when the gutter is
    /// HIDDEN outright: edge-to-edge (no margin to hold it), no buffer name, or a
    /// margin too narrow for even a stub filename ([`rowlayout::GUTTER_MIN_NAME_CHARS`]
    /// — better absent than confetti). The label's right edge lands at `avail` — a
    /// small gap shy of the writing column's left edge — so it hugs the column
    /// from the margin. Shared by [`Self::prepare_gutter`] (what is drawn) and
    /// [`Self::gutter_report`] (what the sidecar says), so the two never drift:
    /// this is the ONE place that decides the gutter's text, never `prepare_gutter`
    /// laying raw text into a wrapping box.
    ///
    /// **Neither line yields to the other from width pressure** — both share the
    /// SAME `avail_chars` budget and elide independently through
    /// [`rowlayout::fit_primary`]; the project line comes back empty here only
    /// when `self.gutter_project` itself is empty (no project at all), never as a
    /// forced yield to protect the filename.
    pub(super) fn gutter_layout(&self) -> Option<GutterLayout> {
        if !crate::page::page_on() || self.gutter_name.is_empty() {
            return None;
        }
        // SUMMONED OVERLAYS OWN THE MARGINS: while ANY overlay is open —
        // every summoned picker, the blurred-backdrop ones AND the CRISP
        // theme/caret/history ones (all `overlay_active`) — the bottom-left
        // orientation gutter yields, returning on dismissal. This generalizes the
        // earlier Bars-only suppression (a Bars takeover drops the boxed card, so
        // the gutter collided with the overlay's foot HINT row): the gutter is
        // redundant orientation noise under ANY summoned surface, and ceding the
        // margin is the lava rail-carve precedent. The ONE gutter-layout owner
        // every reader routes through (draw, frost carve, sidecar `gutter_report`).
        if self.overlay_active {
            return None;
        }
        let gap = self.metrics.char_width * MARGIN_COLUMN_GAP_CHARS.0;
        let avail = self.column_left() - gap;
        // Char budget at the LABEL scale the gutter actually renders at (the doc's
        // own `metrics.char_width` is the FULL-size advance; the gutter's glyphs
        // are smaller, so its per-char footprint shrinks with it).
        let label_char_w = self.metrics.char_width * crate::markdown::type_scale::LABEL;
        let avail_chars = if label_char_w > 0.0 {
            (avail / label_char_w).floor().max(0.0) as usize
        } else {
            0
        };
        let plan = rowlayout::gutter_plan(avail_chars)?;
        let name = rowlayout::fit_primary(&self.gutter_name, plan.name_budget);
        let project = if plan.show_project && !self.gutter_project.is_empty() {
            rowlayout::fit_primary(&self.gutter_project, plan.project_budget)
        } else {
            String::new()
        };
        // The affordance rides the SAME per-line budget and the SAME one elision
        // door as the other two lines — never a third convention — so a narrow
        // margin elides it rather than overflowing the writing column.
        let changed = if self.gutter_changed {
            rowlayout::fit_primary(GUTTER_CHANGED_LABEL, plan.name_budget)
        } else {
            String::new()
        };
        // The working set's rows ride the SAME per-line budget as the filename
        // they widen into, through the same one elision door. Empty in, empty
        // out: `stack_rows` already decided there is no stack to draw, and this
        // does not second-guess it with a count of its own.
        let files = gutter_stack::fit_rows(&self.gutter_files, plan.name_budget);
        Some(GutterLayout {
            avail,
            name,
            project,
            changed,
            files,
        })
    }

    /// Shape + upload the page-mode ORIENTATION GUTTER: a quiet stacked label in the
    /// BOTTOM-LEFT margin — the filename (LABEL size × MUTED ink) over the project (LABEL ×
    /// FAINT ink), RIGHT-aligned so it hugs the writing column from the margin and
    /// anchored to the BOTTOM of the left margin. This relocates orientation OUT of the
    /// writing column into the side (DESIGN §4: the faintest inks at the smallest size,
    /// present when you look, invisible when you don't). HIDDEN edge-to-edge / with no
    /// name (parked off-screen → byte-identical).
    pub(in crate::render) fn prepare_gutter(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let m = self.metrics;
        let label = crate::markdown::type_scale::LABEL;
        let muted = theme::muted().to_glyphon();
        let faint = theme::faint().to_glyphon();
        // A compact stacked label: scale BOTH font size and line height to LABEL so the
        // two rows nest tightly (this buffer is standalone, not row-aligned to the doc).
        self.gutter_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(m.font_size * label, m.line_height * label),
        );
        let base = panel_attrs();
        let bounds = TextBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        // Hidden: empty text parked off-screen, so nothing draws and a non-page (or
        // unnamed) capture stays byte-identical.
        let Some(layout) = self.gutter_layout() else {
            // No block, no stack — drop whatever plate the previous frame left,
            // so a hidden gutter cannot leave a band floating in the margin.
            self.gutter_stack_plate
                .prepare(device, queue, width, height, &[]);
            return self.park_gutter_offscreen(device, queue, bounds, muted);
        };
        // The filename AND the project line are ALREADY fit to one line each by
        // `gutter_layout` (through the shared `rowlayout::fit_primary` door) — this
        // box NEVER lays raw, possibly-overflowing text into a wrapping width, so
        // neither line can ever word-wrap mid-word.
        let lines = layout.lines().len();
        let name = layout.name.clone();
        let project = layout.project.clone();
        // `changed elsewhere` (base content) over filename (muted) over project
        // (faint) — a three-step VALUE ladder with the state at the top of it,
        // since that is the one line here that is news. Each lower line carries
        // its own leading newline so the block stacks; an absent line contributes
        // nothing at all, so an ordinary document's gutter is byte-identical to
        // what it drew before this existed.
        let changed_line = if layout.changed.is_empty() {
            String::new()
        } else {
            format!("{}\n", layout.changed)
        };
        let proj_line = if project.is_empty() {
            String::new()
        } else {
            format!("\n{project}")
        };
        // The WORKING SET's spans, already inked — empty for a single file, which
        // is what sends the identity line below down its original path rather
        // than through a stack of one.
        let stack_ink = gutter_stack::stack_spans(&layout.files, self.gutter_stack_hover);
        let mut spans: Vec<(&str, Attrs)> = Vec::new();
        if !changed_line.is_empty() {
            spans.push((
                changed_line.as_str(),
                base.clone().color(theme::base_content().to_glyphon()),
            ));
        }
        if stack_ink.is_empty() {
            spans.push((name.as_str(), base.clone().color(muted)));
        } else {
            if !project.is_empty() {
                spans.push((project.as_str(), base.clone().color(muted)));
                spans.push(("\n", base.clone().color(muted)));
            }
            for (text, ink) in &stack_ink {
                spans.push((text.as_str(), base.clone().color(*ink)));
            }
        }
        if !proj_line.is_empty() && stack_ink.is_empty() {
            spans.push((proj_line.as_str(), base.clone().color(faint)));
        }
        self.gutter_buffer.set_size(
            &mut self.font_system,
            Some(layout.avail),
            Some(m.line_height * label * lines as f32 + 1.0),
        );
        let default_attrs = base.clone().color(muted);
        self.gutter_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Right),
        );
        self.gutter_buffer
            .shape_until_scroll(&mut self.font_system, false);
        // BOTTOM-anchored in the left margin: the stacked block's BOTTOM edge sits
        // [`super::readout::CANVAS_INSET`] up from the canvas bottom — the SAME
        // named inset the corner readouts use, not a second reading of the same
        // 8px — so `top` is the canvas bottom minus the block's own height. Left
        // 0 with the buffer width == `avail` keeps the right edge a gap shy of the column
        // (horizontal placement unchanged; only the vertical anchor moved top → bottom).
        let stack = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            m.line_height * label,
            lines,
            m.px_physical(super::readout::CANVAS_INSET),
            GUTTER_CARVE_BREATH.0,
        );
        // THE ACTIVE ROW'S PLATE, off the SAME planner rows the glyphs sit on.
        // Empty whenever there is no stack, and an empty prepare leaves the
        // pipeline with zero instances — so a single-file frame issues no draw
        // here at all and stays byte-identical to a pre-stack one.
        let plates = gutter_stack::plate_rects(
            &layout,
            &stack,
            m.char_width * label,
            m.line_height * label * gutter_stack::PLATE_PAD_X.0,
        );
        self.gutter_stack_plate
            .set_color(theme::surface_selected().rgba_bytes());
        self.gutter_stack_plate
            .set_corner(m.px_physical(gutter_stack::PLATE_CORNER_PX));
        self.gutter_stack_plate
            .prepare(device, queue, width, height, &plates);
        let area = TextArea {
            buffer: &self.gutter_buffer,
            left: 0.0,
            top: stack.top,
            scale: 1.0,
            bounds,
            default_color: muted,
            custom_glyphs: &[],
        };
        self.gutter_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon gutter prepare failed: {e:?}"))?;
        Ok(())
    }

    /// Whether the page-mode GUTTER is actually DRAWN this frame — THE one
    /// visibility rule, read straight off [`Self::gutter_layout`]'s own full gate
    /// (page mode + a buffer name + a margin past the hard floor), never a
    /// re-derivation. Exposed for the LAVA gutter corner treatment — the shipped
    /// FROST PILL ([`TextPipeline::lava_gutter_frost_rect`]) and its demoted
    /// hard-carve revert ([`TextPipeline::lava_gutter_carve_rect`]), both in
    /// `render/layers.rs`. Reading the SAME owner `prepare_gutter`/`gutter_report`
    /// share means neither can disagree with what the frame draws.
    pub(in crate::render) fn gutter_visible(&self) -> bool {
        self.gutter_layout().is_some()
    }

    /// THE GUTTER'S LOCAL LAVA CARVE RECT `[left, top, right, bottom]` (px) — the
    /// bounded bottom-left region the lava field vanishes from while the gutter
    /// draws, so its `muted`/`faint` stack sits on flat ground while the REST of
    /// both margins keep the lamp (an ordinary doc goes both-sides — the fix for
    /// the gutter gating the whole-margin carve). Derived from the SAME
    /// [`Self::gutter_layout`] owner `prepare_gutter` lays the block from, so the
    /// carve exactly covers the drawn block:
    ///
    /// * `left = 0`, `right = avail` (the gutter's own box — the filename/project
    ///   are RIGHT-aligned within `[0, avail]`, `avail` a small gap shy of the
    ///   writing column), so the carve spans the block's full horizontal extent.
    /// * `bottom = height` (the block is bottom-anchored [`super::readout::
    ///   CANVAS_INSET`] up), `top = the block top minus a half-row breath` — the
    ///   bottom band the two stacked LABEL rows occupy. `None` when the gutter is
    ///   HIDDEN (nothing to carve).
    ///
    /// The half-row breath and the `+1.0` mirror `prepare_gutter`'s own box; the
    /// [`GUTTER_CARVE_BREATH`] constant names the pad the bounds law reads.
    pub(in crate::render) fn gutter_carve_rect(&self, height: u32) -> Option<[f32; 4]> {
        let layout = self.gutter_layout()?;
        Some(
            crate::render::plan::plan_gutter_stack(
                height as f32,
                layout.avail,
                self.metrics.line_height * crate::markdown::type_scale::LABEL,
                layout.lines().len(),
                self.metrics.px_physical(super::readout::CANVAS_INSET),
                GUTTER_CARVE_BREATH.0,
            )
            .carve,
        )
    }

    /// THE ORGANIC FROST SEEDS for the bottom-left GUTTER (the shipped lava
    /// treatment): the filename + project lines each seed halos `[x0, x1, yc, r]`
    /// (device px) hugging their RIGHT-aligned ink near the column, so they join the
    /// SAME summed field the outline feeds ([`TextPipeline::prepare_lava_layer`]) —
    /// a warm organic whisper under the stack instead of the old full-width
    /// rectangle. Seeds hug the ACTUAL ink (each line's width, right-aligned to
    /// `avail`) rather than the whole `[0, avail]` box. `None`-empty when the gutter
    /// is HIDDEN. Rides the SAME [`Self::gutter_layout`] owner + the shared
    /// [`crate::render::frost_seed_radius`] / [`crate::render::push_text_seeds`] the
    /// outline uses, so both surfaces (and both worlds) seed identically.
    pub(in crate::render) fn gutter_frost_seeds(&self, height: u32) -> Vec<[f32; 4]> {
        let Some(layout) = self.gutter_layout() else {
            return Vec::new();
        };
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        if row_h <= 0.0 {
            return Vec::new();
        }
        let r_row = crate::render::frost_seed_radius(
            row_h,
            crate::lava::FROST_FEATHER_PX,
            self.metrics.zoom,
            self.dpi,
        );
        let skirt =
            crate::lava::frost_px(crate::lava::FROST_FEATHER_PX, self.metrics.zoom, self.dpi);
        let pad_x =
            crate::lava::frost_px(crate::lava::FROST_PILL_PAD_X, self.metrics.zoom, self.dpi);
        // The two stacked LABEL rows, bottom-anchored at the SAME named inset
        // (mirrors `prepare_gutter` / `gutter_carve_rect`, and the corner
        // readouts): name over project. Each line is RIGHT-aligned within
        // `[0, avail]`, so its ink hugs the column at the right edge.
        let stack = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            row_h,
            layout.lines().len(),
            self.metrics.px_physical(super::readout::CANVAS_INSET),
            GUTTER_CARVE_BREATH.0,
        );
        // The gutter's own LABEL advance (its glyphs are the doc advance × LABEL).
        let label_char_w = self.metrics.char_width * label;
        let push_line = |seeds: &mut Vec<[f32; 4]>, text: &str, row: f32| {
            if text.is_empty() {
                return;
            }
            let w = (text.chars().count() as f32 * label_char_w).min(layout.avail);
            let yc = stack.rows[row as usize][1] + row_h * 0.5;
            crate::render::push_text_seeds(
                seeds,
                layout.avail - w - pad_x,
                w + 2.0 * pad_x,
                yc,
                r_row,
                skirt,
                text,
            );
        };
        let mut seeds = Vec::new();
        for (row, (text, _)) in layout.lines().into_iter().enumerate() {
            push_line(&mut seeds, text, row as f32);
        }
        seeds
    }

    /// THE ACTIVE STACK ROW'S PLATE RECT `[x, y, w, h]`, off the EXACT SAME
    /// layout + planner rows [`Self::prepare_gutter`] draws
    /// `gutter_stack_plate` from — `None` when nothing is plated (a
    /// single-file margin, or the gutter itself hidden/off).
    ///
    /// Exists for real-pixel laws that need to sample INSIDE the plate without
    /// re-deriving its padding arithmetic by hand (`render/tests/one_bit.rs`'s
    /// stack-plate legibility law): a rect computed any differently than what
    /// production actually filled would defeat the point of testing pixels —
    /// the same reasoning [`Self::gutter_frost_seeds`] already documents for
    /// itself, one door over.
    #[cfg(test)]
    pub(in crate::render) fn gutter_stack_plate_rect(&self, height: u32) -> Option<[f32; 4]> {
        let layout = self.gutter_layout()?;
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        if row_h <= 0.0 {
            return None;
        }
        let stack = crate::render::plan::plan_gutter_stack(
            height as f32,
            layout.avail,
            row_h,
            layout.lines().len(),
            self.metrics.px_physical(super::readout::CANVAS_INSET),
            GUTTER_CARVE_BREATH.0,
        );
        let label_char_w = self.metrics.char_width * label;
        gutter_stack::plate_rects(
            &layout,
            &stack,
            label_char_w,
            row_h * gutter_stack::PLATE_PAD_X.0,
        )
        .into_iter()
        .next()
    }

    /// The page-mode GUTTER state for the capture sidecar: `Some((name, project))`
    /// EXACTLY when the gutter is drawn (page mode on, a buffer name, a margin past
    /// the hard floor — the same gate as [`Self::prepare_gutter`]), else `None`.
    /// Both `name` and `project` are EXACTLY as drawn — each already fit to one
    /// line, independently middle-elided (extension preserved) only once the
    /// margin can't hold it whole. Neither one yields to the other from width
    /// pressure: `project` is empty here only when there is genuinely no project
    /// to show, so the sidecar always agrees with the pixels.
    pub fn gutter_report(&self) -> Option<(String, String, bool)> {
        self.gutter_layout()
            .map(|g| (g.name, g.project, !g.changed.is_empty()))
    }
}
