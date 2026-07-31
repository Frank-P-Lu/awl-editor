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
const RAIL_GAP_CHARS: f32 = 3.0;

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

/// How much presence the UNFOCUSED region's marker keeps, as a fraction of the
/// focused one's alpha. It is the same rect in the same place — only its
/// insistence changes, which is figure/ground by value rather than a second
/// decoration (DESIGN.md §5).
pub(in crate::render) const UNFOCUSED_MARK_ALPHA: f32 = 0.34;

/// Scale a theme colour's ALPHA only, leaving its hue and value alone.
pub(in crate::render) fn dimmed(color: theme::Srgb, f: f32) -> [u8; 4] {
    let mut rgba = color.rgba_bytes();
    rgba[3] = (rgba[3] as f32 * f.clamp(0.0, 1.0)).round().clamp(0.0, 255.0) as u8;
    rgba
}

impl TextPipeline {
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

    /// MEASURE the navigation rail's column width (device px) from the rail's own
    /// shaped labels — the same `&mut FontSystem` measurement item 51 already
    /// makes for a content-hugging card, and for the same reason: a
    /// character-count estimate over a proportional display face is not a width.
    /// Cached into `workspace_rail_w` at `set_view`, so the geometry stays `&self`
    /// and the drawn column, the clip and the hit band all read one number.
    pub(in crate::render) fn measure_workspace_rail_w(&mut self) -> f32 {
        if self.overlay_lens.is_empty() {
            return 0.0;
        }
        self.overlay_remetric();
        let ui_metrics = self.overlay_metrics();
        self.workspace_rail_buffer
            .set_metrics(&mut self.font_system, ui_metrics);
        self.workspace_rail_buffer
            .set_size(&mut self.font_system, None, None);
        self.workspace_rail_buffer
            .set_wrap(&mut self.font_system, Wrap::None);
        let text = self
            .overlay_lens
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let ink = theme::base_content().to_glyphon();
        self.workspace_rail_buffer.set_text(
            &mut self.font_system,
            &text,
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
        widest + 2.0 * self.overlay_text_hpad()
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
        self.workspace_rail_w > 0.0
            && interior - self.workspace_rail_w - RAIL_GAP_CHARS * cw >= MIN_PANE_CHARS * cw
    }

    /// THE WORKSPACE'S GEOMETRY — the third overlay family, beside the flat
    /// pickers and the grouped/faceted card. It is deliberately not a variant of
    /// either: a workspace's box comes from the canvas rather than from a width
    /// cap and an anchor rail, because it is not a card seeking a comfortable
    /// place to float.
    pub(in crate::render) fn workspace_geometry(&self, width: u32) -> OverlayGeom {
        let lh = self.overlay_lh();
        let cw = self.overlay_char_width();
        let pad = WORKSPACE_PAD;
        let margin = self.workspace_margin();
        let n_items = self.overlay_items.len();

        // THE FOOTER'S HOME IS THE REGION THAT IS SHOWING ITS LIST. With a content
        // pane on screen the hint is that pane's footer, under the rows, exactly
        // as every picker's is. On the narrow PRIMARY stage there are no rows —
        // the rail is the list — so the hint follows the rail instead of hanging
        // in an empty pane a full viewport above the eye that needs it. One
        // sentence, in one place, either way; the rail shaper reads this same
        // `hint_rows == 0 && rail.is_some()` fact rather than a second flag.
        let show_rows = self.overlay_detail_focus || self.workspace_is_wide(width);
        let hint = self.overlay_hint.clone();
        let hint_rows = usize::from(!hint.is_empty() && show_rows);
        let empty = if n_items == 0 {
            self.overlay_empty.clone()
        } else {
            None
        };
        let empty_rows = empty.is_some() as usize;
        let header_rows = 1; // the `settings › query` search line
        let header_gap = self.overlay_header_gap();

        let card_x = margin;
        let card_w = (width as f32 - 2.0 * margin).max(0.0);
        let card_y = margin + self.menubar_reserve();
        let card_h = (self.window_h - card_y - margin).max(lh);

        // ── THE TWO REGIONS ───────────────────────────────────────────────
        // The rail wants its measured column; the content wants everything
        // else. When "everything else" is no longer a legible pane, the
        // workspace stages the two instead of squeezing them.
        let hpad = self.overlay_text_hpad();
        let rail_w = self.workspace_rail_w;
        let gap = RAIL_GAP_CHARS * cw;
        let interior = (card_w - 2.0 * hpad).max(0.0);
        let wide = self.workspace_is_wide(width);
        let rows_focused = self.overlay_detail_focus;

        // The rail carries its COLUMN only (`[x, w]`). Its vertical grid is
        // resolved from the row plan's own band origin at draw and hit time
        // (`workspace_rail_box`), so a rail entry and the settings row beside it
        // sit on one line by construction rather than by two arithmetics
        // agreeing — the same rule item 174 wrote for the candidate rows.
        let (rail, pane_x, pane_w) = match (wide, rows_focused) {
            // Wide: the rail column, then the content pane beside it.
            (true, _) => (
                Some([card_x + hpad, rail_w]),
                card_x + hpad + rail_w + gap,
                (card_w - 2.0 * hpad - rail_w - gap).max(0.0),
            ),
            // Narrow, primary stage: the rail IS the workspace.
            (false, false) => (Some([card_x + hpad, interior]), card_x + hpad, interior),
            // Narrow, detail stage: the content is.
            (false, true) => (None, card_x + hpad, interior),
        };

        // On the narrow PRIMARY stage the content pane draws no rows at all — the
        // rail is the list you are on. The search line still rides above it:
        // typing is how you search from either stage, and a field you cannot see
        // is a field you will not use.
        let avail_px = (card_h - 2.0 * pad - header_gap).max(lh);
        let chrome_rows = header_rows + hint_rows + empty_rows;
        let (top_idx, visible) = match show_rows {
            true => self.overlay_flat_window(n_items, avail_px, chrome_rows),
            false => (0, 0),
        };

        OverlayGeom {
            visible,
            top_idx,
            n_items,
            hint: if show_rows { hint } else { String::new() },
            hint_rows,
            header_rows,
            header_gap,
            empty: show_rows.then_some(empty).flatten(),
            card_x,
            card_y,
            card_w,
            card_h,
            // The search line and the rows both live in the content pane, so the
            // shaper's text box IS that pane.
            text_left: pane_x,
            text_top: card_y + pad,
            text_w: pane_w,
            // A workspace is never in the card's fill regime: it already fills.
            // Saying so explicitly keeps the placard's own narrow-card rule
            // (`overlay_shape_placard`) reading a fact rather than a coincidence.
            card_narrow: false,
            workspace: true,
            rail,
            pane_x,
            pane_w,
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
            self.workspace_rail_area = None;
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
        self.workspace_rail_area = Some((x, top));
        self.workspace_rail_mark = active.and_then(|i| self.workspace_rail_rect(geom, plan, i));
        true
    }
}
