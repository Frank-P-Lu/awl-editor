//! THE SUMMONED WORKSPACE'S PRESENTATION (queue item 114) — geometry, the
//! navigation rail's shaping and marks, and the rail's pointer hit-test.
//!
//! A contextual overlay is a card floating over a document the user still needs
//! to see. A summoned WORKSPACE relocates attention: it takes the viewport,
//! leaves the document as a quiet backdrop framing it, and gives one sustained
//! task two coordinated regions (DESIGN.md §5). The content model — which kinds
//! get this shell, what the rail lists, which region holds focus — is
//! `crate::overlay::workspace`; this file is only how that is drawn and pointed
//! at.
//!
//! # Wide and narrow are ONE fact, read two ways
//!
//! The lifecycle's focus stage (`Surface::Workspace` = the rail,
//! `Surface::WorkspaceDetail` = the content) is width-blind by construction —
//! item 173 kept width out of `landing_of` so Back could not branch on it. This
//! file is where width finally enters, and it enters once, in
//! [`TextPipeline::workspace_geometry`]:
//!
//!   * **WIDE** — both regions are drawn side by side; the focus stage says which
//!     one carries the world's full selected-row band and which carries the same
//!     rect at reduced presence.
//!   * **NARROW** — only the focused region is drawn, and the same focus stage is
//!     now *which stage you are on*. `Esc` still means "back to the primary
//!     list", because the table it comes from cannot see the width (DESIGN.md §8:
//!     narrow layouts stage, they do not miniaturize).
//!
//! Nothing else in the tree branches on the workspace's width, and no lifecycle
//! transition can, which is what makes the two presentations one behaviour.

use super::*;

/// The workspace's inset from the window edge, as a fraction of the smaller
/// window dimension. Generous enough that the document reads as a quiet frame
/// around the workspace rather than being erased by it, small enough that
/// attention has plainly moved.
const WORKSPACE_MARGIN_FRAC: f32 = 0.055;
const WORKSPACE_MARGIN_MIN: f32 = 14.0;
const WORKSPACE_MARGIN_MAX: f32 = 72.0;

/// The breathing room between the rail column and the content pane, in overlay
/// character widths — the same currency `rowlayout::GAP_CHARS` spends between a
/// row's primary and secondary cells, so the workspace's internal rhythm is the
/// picker's own.
pub(in crate::render) const RAIL_GAP_CHARS: f32 = 3.0;

/// The workspace's inner padding, between its own edge and its regions.
pub(in crate::render) const WORKSPACE_PAD: f32 = 12.0;

/// The narrowest content pane that still deserves a rail beside it, in overlay
/// character widths — wide enough for a row's NAME and its VALUE together, with
/// the gap `rowlayout` puts between them.
///
/// This is the whole of the wide/narrow decision, and it is a LEGIBILITY floor
/// rather than a device breakpoint: below it the secondary column starts
/// yielding (`rowlayout` drops it first under width pressure), and a settings
/// list that shows what a setting is called but not what it is set to is not a
/// settings list. DESIGN.md §8 rule 4 says to stage the regions at that point
/// rather than compress them, which is exactly what falling below this does.
const MIN_PANE_CHARS: f32 = 46.0;

/// ITEM 116d — THE TIMELINE COLUMN'S WIDTH POLICY, and why it is not the rail's.
///
/// A rail of CATEGORY LABELS is measured exactly: it holds a fixed vocabulary of
/// display-face words and must hold the widest of them or a category reads
/// truncated. A TIMELINE is a list of ROWS — `2 hr ago · edited "…"` — that
/// already elide through `rowlayout`, and whose widest member is whatever the
/// longest commit subject in the file's history happens to be. Measuring that
/// exactly would let one verbose commit take the comparison's width away, which
/// is the opposite of DESIGN.md §5's "a narrow timeline beside a large
/// comparison". So the timeline HUGS its content (item 51's pattern) and is then
/// BOUNDED: it can never grow past this fraction of the workspace's interior, and
/// it can never COLLAPSE below a floor that keeps a `yesterday 12:34` label
/// readable — the case a bare hug gets wrong is an empty history, whose corpus is
/// one notice rather than a list.
const TIMELINE_MIN_CHARS: f32 = 12.0;
const TIMELINE_MAX_FRAC: f32 = 0.34;

/// How much presence the UNFOCUSED region's marker keeps, as a fraction of the
/// focused one's alpha. It is the same rect in the same place — only its
/// insistence changes, which is figure/ground by value rather than a second
/// decoration (DESIGN.md §5).
pub(in crate::render) const UNFOCUSED_MARK_ALPHA: f32 = 0.34;

/// Scale a theme colour's ALPHA only, leaving its hue and value alone.
pub(in crate::render) fn dimmed(color: theme::Srgb, f: f32) -> [u8; 4] {
    let mut rgba = color.rgba_bytes();
    rgba[3] = (rgba[3] as f32 * f.clamp(0.0, 1.0))
        .round()
        .clamp(0.0, 255.0) as u8;
    rgba
}

impl OverlayGeom {
    /// The CONTENT BAND's horizontal extent — the one owner every row-band
    /// consumer (the plan, the selected-row quads, the bar plates, the pointer
    /// hit-test) reads. It is the card for a contextual overlay and the content
    /// pane for a workspace, so none of those consumers has to know which it is.
    pub(super) fn band_x(&self) -> f32 {
        match self.workspace {
            true => self.pane_x,
            false => self.card_x,
        }
    }

    pub(super) fn band_w(&self) -> f32 {
        match self.workspace {
            true => self.pane_w,
            false => self.card_w,
        }
    }

    /// TEST-ONLY readers for the item-114 law probe (`render/tests/overlay_probe.rs`),
    /// which lives outside this module so a law can compare against what the
    /// frame committed without a render path growing an exception.
    #[cfg(test)]
    pub(in crate::render) fn band_x_probe(&self) -> f32 {
        self.band_x()
    }

    #[cfg(test)]
    pub(in crate::render) fn band_w_probe(&self) -> f32 {
        self.band_w()
    }

    #[cfg(test)]
    pub(in crate::render) fn card_probe(&self) -> [f32; 4] {
        [self.card_x, self.card_y, self.card_w, self.card_h]
    }

    #[cfg(test)]
    pub(in crate::render) fn visible_probe(&self) -> usize {
        self.visible
    }
}

impl TextPipeline {
    /// ITEM 114 — WHERE the navigation rail's shaped labels go, and the clip
    /// that keeps them there: its measured column, so a label can never bleed
    /// into the content pane it sits beside. Returns placement rather than a
    /// `TextArea` so the caller's own field borrows stay disjoint from the
    /// renderer it is about to prepare.
    pub(super) fn workspace_rail_area(
        &self,
        geom: &OverlayGeom,
        width: u32,
        height: u32,
    ) -> Option<(f32, f32, TextBounds)> {
        let (left, top) = self.workspace_rail_placement?;
        let rail_w = geom.rail.map(|[_, w]| w).unwrap_or(0.0);
        Some((
            left,
            top,
            TextBounds {
                left: left.max(0.0) as i32,
                top: 0,
                right: ((left + rail_w).min(width as f32)) as i32,
                bottom: height as i32,
            },
        ))
    }

    /// ITEM 114 — THE RAIL'S ACTIVE MARK, in the shared "active lens mark" slot,
    /// because that is exactly what it is: the rail IS the facet strip stood on
    /// its end, and its active entry is that strip's active label.
    ///
    /// It bypasses the `FacetStyle` skins on purpose — those describe a
    /// horizontal chip run (a bracket, an underline hugging a baseline) and none
    /// of them says anything about a column — and takes the world's own
    /// selected-row band instead, at the same reduced presence the content pane's
    /// band takes when IT is the unfocused region.
    pub(super) fn prepare_rail_mark(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
    ) {
        let band = match theme::active()
            .highlight_treatment(crate::render::effective_overlay_selrow_band())
        {
            theme::HighlightTreatment::ValueBand(c) => c,
            theme::HighlightTreatment::InverseFill { band, .. } => band,
        };
        let rgba = match geom.rows_focused {
            true => super::workspace::dimmed(band, super::workspace::UNFOCUSED_MARK_ALPHA),
            false => band.rgba_bytes(),
        };
        self.overlay_lens_underline.set_color(rgba);
        self.overlay_lens_underline
            .set_corner(super::overlay_rows::FACET_CHIP_RADIUS);
        self.overlay_lens_underline.set_stroke(0.0);
        let marks: Vec<[f32; 4]> = self.workspace_rail_mark.into_iter().collect();
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &marks);
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &[]);
    }

    /// The workspace inset for the current window — a pure function of the
    /// canvas, read by the geometry and by the laws that check it.
    pub(in crate::render) fn workspace_margin(&self) -> f32 {
        let smaller = self.window_w.min(self.window_h).max(0.0);
        (smaller * WORKSPACE_MARGIN_FRAC).clamp(WORKSPACE_MARGIN_MIN, WORKSPACE_MARGIN_MAX)
    }

    /// Is the summoned card drawn as a workspace this frame?
    pub(in crate::render) fn overlay_is_workspace(&self) -> bool {
        self.overlay_workspace && !self.overlay_lens.is_empty()
    }

    /// MEASURE the workspace's PRIMARY (narrow) column width (device px) from its
    /// own shaped content — the same `&mut FontSystem` measurement item 51 already
    /// makes for a content-hugging card, and for the same reason: a
    /// character-count estimate over a proportional display face is not a width.
    /// Cached into `workspace_primary_w` at `set_view`, so the geometry stays
    /// `&self` and the drawn column, the clip and the hit band all read one number.
    ///
    /// WHICH content it measures is item 116a's one fact: a `RailOverRows`
    /// workspace's primary column carries category LABELS, a
    /// `TimelineOverComparison` one carries the workspace's own ROWS. Both are
    /// shaped through the same buffer with the same attrs; only the corpus and the
    /// bound differ (see [`TIMELINE_MAX_FRAC`]).
    pub(in crate::render) fn measure_workspace_primary_w(&mut self) -> f32 {
        let rows_primary = self.overlay_rows_primary;
        if self.overlay_lens.is_empty() {
            return 0.0;
        }
        let text = match rows_primary {
            // The EMPTY-STATE notice is part of the corpus: an empty history's
            // column has to hold "no history yet" as surely as a full one holds
            // its widest row.
            true => match self.overlay_items.is_empty() {
                true => self.overlay_empty.clone().unwrap_or_default(),
                false => self.overlay_items.join("\n"),
            },
            false => self
                .overlay_lens
                .iter()
                .map(|(label, _)| label.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let widest = self.measure_workspace_column_px(&text);
        let hpad = self.overlay_text_hpad();
        if !rows_primary {
            return widest + 2.0 * hpad;
        }
        // THE FOOTER IS PART OF THE COLUMN'S CONTENT, so it is part of the
        // measurement. On the rail shape the footer rides the wide pane beside the
        // rail, so its width is never the rail's problem; on the timeline shape it
        // rides the timeline itself, and a column measured from the rows alone
        // clips the very line that teaches the keys — the exact failure the last
        // slice's vision smoke caught on a footer running off a card. It is shaped
        // at the UI face here and scaled by `LABEL`, which is the size the footer
        // really draws at (`push_overlay_hint_spans`).
        let hint = match self.overlay_hint.is_empty() {
            true => 0.0,
            false => {
                self.measure_workspace_column_px(&self.overlay_hint.clone())
                    * crate::markdown::type_scale::LABEL
            }
        };
        // A timeline is BOUNDED, never merely measured — and it keeps its floor
        // even when the corpus is empty ("no history yet"), so an empty timeline
        // is still a timeline beside a comparison rather than a collapsed column.
        let cw = self.overlay_char_width();
        let interior = (self.window_w - 2.0 * self.workspace_margin() - 2.0 * hpad).max(0.0);
        let floor = TIMELINE_MIN_CHARS * cw;
        (widest.max(hint) + 2.0 * hpad).clamp(floor, (interior * TIMELINE_MAX_FRAC).max(floor))
    }

    /// Shape `text` into the primary column's own buffer and report its widest
    /// line. The one shaping seam both corpora above go through.
    fn measure_workspace_column_px(&mut self, text: &str) -> f32 {
        self.overlay_remetric();
        let ui_metrics = self.overlay_metrics();
        self.workspace_rail_buffer
            .set_metrics(&mut self.font_system, ui_metrics);
        self.workspace_rail_buffer
            .set_size(&mut self.font_system, None, None);
        self.workspace_rail_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let ink = theme::base_content().to_glyphon();
        self.workspace_rail_buffer.set_text(
            &mut self.font_system,
            text,
            &panel_attrs().color(ink),
            Shaping::Advanced,
            None,
        );
        self.workspace_rail_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let mut widest = 0.0_f32;
        for run in self.workspace_rail_buffer.layout_runs() {
            widest = widest.max(run.line_w);
        }
        widest
    }

    /// TEST-ONLY readers for the item-114 law probe.
    #[cfg(test)]
    pub(in crate::render) fn overlay_lens_len(&self) -> usize {
        self.overlay_lens.len()
    }

    #[cfg(test)]
    pub(in crate::render) fn workspace_rail_mark_probe(&self) -> Option<[f32; 4]> {
        self.workspace_rail_mark
    }

    /// IS THERE ROOM FOR BOTH REGIONS AT ONCE? The one width decision the whole
    /// workspace makes, and the only place width enters this feature at all
    /// (see the module doc). `true` draws the rail beside the content; `false`
    /// stages them, and the lifecycle's focus stage becomes which stage you are
    /// on — with no arm of the transition table able to tell the difference.
    pub(in crate::render) fn workspace_is_wide(&self, width: u32) -> bool {
        let hpad = self.overlay_text_hpad();
        let interior = (width as f32 - 2.0 * self.workspace_margin() - 2.0 * hpad).max(0.0);
        let cw = self.overlay_char_width();
        self.workspace_primary_w > 0.0
            && interior - self.workspace_primary_w - RAIL_GAP_CHARS * cw >= MIN_PANE_CHARS * cw
    }

    /// HOW MANY DISPLAY LINES A WORKSPACE DRAWS ABOVE ITS CANDIDATE BAND — one
    /// owner, because two consumers need it and they cannot be allowed to
    /// disagree: [`Self::workspace_geometry`], which plans them, and
    /// [`Self::comparison_viewport`], which opens the relocated document on the
    /// line they close.
    ///
    /// A `RailOverRows` workspace draws one — its `settings › query` search line —
    /// with its LENS living in the primary column as a rail of labels. A
    /// `TimelineOverComparison` workspace has no label rail to put a lens in (its
    /// primary column carries the timeline itself), so the lens moves into the
    /// HEADER as a second line, exactly the grouped card's own composition.
    pub(in crate::render) fn workspace_header_rows(&self) -> usize {
        1 + usize::from(self.overlay_rows_primary)
    }

    /// THE WORKSPACE'S GEOMETRY — the third overlay family, beside the flat
    /// pickers and the grouped/faceted card. It is deliberately not a variant of
    /// either: a workspace's box comes from the canvas rather than from a width
    /// cap and an anchor rail, because it is not a card seeking a comfortable
    /// place to float.
    pub(in crate::render) fn workspace_geometry(&self, width: u32) -> OverlayGeom {
        let lh = self.overlay_lh();
        let pad = WORKSPACE_PAD;
        let n_items = self.overlay_items.len();
        // ITEM 116b — the POSITIONAL half lives in `comparison.rs`, so the row
        // geometry below and the relocated document viewport read ONE
        // derivation of the card box, the primary column and the content pane.
        let regions = self.workspace_regions(width);
        let rows_focused = regions.content_focused;

        // ITEM 116a — THE ONE FACT THE TWO REGIONS' ROLES REDUCE TO.
        // `RailOverRows` (Settings, today) keeps the row list in the CONTENT
        // pane behind a PRIMARY column of labels; `TimelineOverComparison`
        // (item 116d, unreached) keeps it in the PRIMARY column instead,
        // behind a content region this module never draws into — item 116b's
        // `comparison_viewport`, which the document layer itself relocates to.
        // `primary_visible`/`content_visible` are which REGION is on screen —
        // unchanged by the shape; `show_rows` is whichever owns the rows.
        let rows_primary = self.overlay_rows_primary;
        let primary_visible = regions.primary_visible();
        let content_visible = regions.content_visible();
        let show_rows = if rows_primary {
            primary_visible
        } else {
            content_visible
        };

        // THE FOOTER FOLLOWS WHICHEVER REGION IS SHOWING ITS LIST — the rail
        // shaper reads this same `hint_rows == 0 && rail.is_some()` fact
        // rather than a second flag, so one sentence lives in one place.
        let hint = self.overlay_hint.clone();
        let hint_rows = usize::from(!hint.is_empty() && show_rows);
        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;
        // The `settings › query` search line, plus — when the primary column
        // carries the rows — the LENS STRIP that has nowhere else to live.
        let header_rows = self.workspace_header_rows();
        let header_gap = self.overlay_header_gap();

        let [card_x, card_y, card_w, card_h] = regions.card;
        let ([primary_x, primary_w], [pane_x, pane_w]) = (regions.primary, regions.pane);

        // A LABEL RAIL is shaped only when the primary column shows LABELS
        // (`!rows_primary`) and only while visible; its grid is resolved from
        // the row plan's own band origin at draw/hit time (`workspace_rail_box`).
        // When rows are primary there is no label list to shape — the rows
        // fill that column through `text_left`/`text_w` below instead, just as
        // the content pane's rows do today.
        let rail = (!rows_primary && primary_visible).then_some([primary_x, primary_w]);

        // THE ROW LIST'S OWN BOX follows `rows_primary`: the primary column's
        // when it owns the rows, the content pane's otherwise — today's rule,
        // and the only one any kind currently reaches.
        let (text_left, text_w) = if rows_primary {
            (primary_x, primary_w)
        } else {
            (pane_x, pane_w)
        };

        // On a stage that is not showing its list, no rows are windowed at
        // all — the search line still rides above at `text_left`/`text_top`:
        // typing is how you search from either stage, and a field you cannot
        // see is a field you will not use.
        let avail_px = (card_h - 2.0 * pad - header_gap).max(lh);
        let chrome_rows = header_rows + hint_rows + empty_rows;
        let (top_idx, visible) = match show_rows {
            true => self.overlay_flat_window(n_items, avail_px, chrome_rows),
            false => (0, 0),
        };

        // ITEM 116d — A LENS IN THE HEADER IS THE GROUPED CARD'S OWN COMPOSITION,
        // so it takes the grouped card's own shaper rather than a second one.
        // `theme` here means "this card draws a lens STRIP on its last header
        // line", which is exactly what a `TimelineOverComparison` workspace needs
        // and exactly what the grouped family already owns end to end (shaping,
        // the active mark, the strip's own pointer hit-test). The display-line
        // sequence it consumes is the FLAT window this geometry already resolved
        // — a timeline has no section headers — so the plan, the shaped lines and
        // the hit-test stay one object.
        let strip_in_header = rows_primary;
        let lines: Vec<PlanLine> = match strip_in_header {
            true => (top_idx..top_idx + visible).map(PlanLine::Item).collect(),
            false => Vec::new(),
        };

        OverlayGeom {
            visible,
            top_idx,
            n_items,
            hint: if show_rows { hint } else { String::new() },
            hint_rows,
            header_rows,
            header_gap,
            theme: strip_in_header,
            strip: match strip_in_header {
                true => self.overlay_lens.clone(),
                false => Vec::new(),
            },
            plan: lines,
            empty: show_rows.then_some(empty).flatten(),
            card_x,
            card_y,
            card_w,
            card_h,
            text_left,
            text_top: card_y + pad,
            text_w,
            // A workspace is never in the card's fill regime: it already fills.
            // Saying so explicitly keeps the placard's own narrow-card rule
            // (`overlay_shape_placard`) reading a fact rather than a coincidence.
            card_narrow: false,
            workspace: true,
            rail,
            // THE CONTENT BAND IS THE ROW LIST'S BOX, not "the wide region": every
            // band consumer (the row plan, the selected-row quads, the bar plates,
            // the pointer hit-test) reads `band_x`/`band_w`, and on the timeline
            // shape the rows live in the PRIMARY column. Pointing this at the
            // comparison instead would draw the selected-version band across the
            // transcript and make the timeline clickable nowhere it is drawn.
            pane_x: text_left,
            pane_w: text_w,
            rows_focused,
            ..OverlayGeom::base()
        }
    }

    /// The rail's full box `[x, y, w, h]` — its measured column, seated on the
    /// row plan's own band origin so its first entry and the pane's first row
    /// share a line, and running to the workspace's bottom pad.
    pub(in crate::render) fn workspace_rail_box(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> Option<[f32; 4]> {
        let [x, w] = geom.rail?;
        let top = plan.first_top();
        let bottom = geom.card_y + geom.card_h - WORKSPACE_PAD;
        (bottom > top).then_some([x, top, w, bottom - top])
    }

    /// The rect of rail entry `idx`, or `None` when no rail is drawn or the entry
    /// would fall outside the rail's own box.
    pub(in crate::render) fn workspace_rail_rect(
        &self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
        idx: usize,
    ) -> Option<[f32; 4]> {
        let [x, top, w, h] = self.workspace_rail_box(geom, plan)?;
        let lh = self.overlay_lh();
        let row_top = top + idx as f32 * lh;
        (row_top + lh <= top + h + 0.5).then_some([x, row_top, w, lh])
    }

    /// HIT-TEST the navigation rail: which rail entry does `(px, py)` select?
    /// `None` off the rail, off the card, or when this card has no rail — which
    /// is what stops a pointer in the rail column from ever resolving to a
    /// settings row (that lookup is bounded to `geom.pane_x`/`pane_w`).
    pub fn workspace_rail_at(&self, px: f32, py: f32) -> Option<usize> {
        if !self.overlay_active || !self.overlay_is_workspace() {
            return None;
        }
        let geom = self.workspace_geometry(self.window_w as u32);
        let plan = self.overlay_row_plan(&geom);
        let [x, _, w, _] = self.workspace_rail_box(&geom, &plan)?;
        if px < x || px >= x + w {
            return None;
        }
        (0..self.overlay_lens.len()).find(|&i| {
            self.workspace_rail_rect(&geom, &plan, i)
                .is_some_and(|[_, ry, _, rh]| py >= ry && py < ry + rh)
        })
    }

    /// SHAPE the rail's labels into their own buffer and record the ACTIVE
    /// entry's mark rect. Returns whether a rail was shaped at all.
    ///
    /// The active entry takes content ink and the rest muted, the same
    /// figure/ground grammar every picker row uses; the mark rect is handed to
    /// [`super::overlay_rows`]'s facet-mark owner rather than drawn here, so the
    /// rail's band and the content pane's band come out of the same treatment.
    pub(in crate::render) fn workspace_shape_rail(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> bool {
        self.workspace_rail_mark = None;
        let Some([x, top, w, _]) = self.workspace_rail_box(geom, plan) else {
            self.workspace_rail_placement = None;
            return false;
        };
        let ui_metrics = self.overlay_metrics();
        self.workspace_rail_buffer
            .set_metrics(&mut self.font_system, ui_metrics);
        self.workspace_rail_buffer
            .set_size(&mut self.font_system, Some(w), None);
        self.workspace_rail_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let content = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let base = panel_attrs();
        let faint = theme::faint().to_glyphon();
        let mut spans: Vec<(String, glyphon::Color)> = Vec::new();
        let mut active: Option<usize> = None;
        for (i, (label, is_active)) in self.overlay_lens.iter().enumerate() {
            if *is_active {
                active = Some(i);
            }
            let line = match i {
                0 => label.clone(),
                _ => format!("\n{label}"),
            };
            spans.push((line, if *is_active { content } else { muted }));
        }
        // The footer follows the list (see `workspace_geometry`): when the pane
        // is drawing no rows it carries no hint either, so the rail carries it,
        // one blank line below its last category.
        if geom.hint_rows == 0 && !self.overlay_hint.is_empty() {
            spans.push((format!("\n\n{}", self.overlay_hint), faint));
        }
        let rich: Vec<(&str, Attrs)> = spans
            .iter()
            .map(|(s, c)| (s.as_str(), base.clone().color(*c)))
            .collect();
        let default_attrs = base.clone().color(muted);
        self.workspace_rail_buffer.set_rich_text(
            &mut self.font_system,
            rich,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.workspace_rail_buffer
            .shape_until_scroll(&mut self.font_system, false);
        self.workspace_rail_placement = Some((x, top));
        self.workspace_rail_mark = active.and_then(|i| self.workspace_rail_rect(geom, plan, i));
        true
    }
}
