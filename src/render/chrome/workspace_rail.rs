//! THE SUMMONED WORKSPACE'S NAVIGATION RAIL — its grid, its shaping, its
//! active mark, and its pointer hit-test.
//!
//! Its sibling `workspace.rs` owns the workspace's geometry and regions; this
//! file owns the narrow one of those regions, the same way `workspace_column.rs`
//! owns how wide it is.
//!
//! # A rail is a LIST
//!
//! That is the whole reason this file has a composition question in it at all.
//! The rail is the facet strip stood on its end, and its active entry is that
//! strip's active label — so it bypasses the `FacetStyle` skins, which describe
//! a horizontal chip run and say nothing about a column, and takes the world's
//! own LIST composition instead. Every style but one answers with the filled
//! selected-row band. `ListStyle::Ruled` cannot: a filled band is the one thing
//! that composition refuses, and taking it here shipped a world whose content
//! pane was arranged by rules while the rail beside it wore a plate. The `Ruled`
//! arm therefore routes through the same owner the rows come out of
//! (`overlay_rules::rules_ink`), on the two pipelines the chip skins otherwise
//! use — hairlines on `overlay_facet_ghost`, the selection on
//! `overlay_lens_underline`.
//!
//! Either way the two regions still differ only in PRESENCE
//! (`UNFOCUSED_MARK_ALPHA`), never in decoration: the same mark in the same
//! place, insisting less (DESIGN.md §5).

use super::workspace::WORKSPACE_PAD;
use super::*;

impl TextPipeline {
    /// WHERE the navigation rail's shaped labels go, and the clip
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

    /// THE RAIL'S ACTIVE MARK, in the shared "active lens mark" slot,
    /// because that is exactly what it is: the rail IS the facet strip stood on
    /// its end, and its active entry is that strip's active label.
    ///
    /// It bypasses the `FacetStyle` skins on purpose — those describe a
    /// horizontal chip run (a bracket, an underline hugging a baseline) and none
    /// of them says anything about a column. What it takes instead is the
    /// world's own list composition, because a rail IS a list: a filled band on
    /// a `Pane`/`Bars`/`Diagonal` world, and the same rules the rows beside it
    /// are arranged by on a `Ruled` one. Either way at the same reduced presence
    /// the content pane's mark takes when IT is the unfocused region.
    pub(super) fn prepare_rail_mark(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
    ) {
        // A RAIL IS A LIST, so the style that says how a list is arranged says
        // how this one is. Only `Ruled` answers differently, because only
        // `Ruled` refuses the filled band every other style's mark is made of;
        // no wildcard, so a fifth style has to decide.
        match crate::render::effective_list_style() {
            theme::ListStyle::Ruled(mark) => {
                self.prepare_rail_rules(device, queue, width, height, geom, mark);
                return;
            }
            theme::ListStyle::Pane | theme::ListStyle::Bars | theme::ListStyle::Diagonal(_) => {}
        }
        // The fill and the ink that reads on it are ONE decision, taken in one
        // place (`overlay_visual_sel`); this arm takes the fill and
        // `workspace_shape_rail` takes its ink from the same pair.
        let band = super::overlay_selected_band_srgb();
        let rgba = match geom.rows_focused {
            true => super::workspace::dimmed(band, super::workspace::UNFOCUSED_MARK_ALPHA),
            false => band.rgba_bytes(),
        };
        self.overlay_lens_underline.set_color(rgba);
        self.overlay_lens_underline
            .set_corner(self.metrics.px(super::overlay_rows::FACET_CHIP_RADIUS));
        self.overlay_lens_underline.set_stroke(0.0);
        let marks: Vec<[f32; 4]> = self.workspace_rail_mark().into_iter().collect();
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &marks);
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &[]);
    }

    /// THE RAIL AS A RULED LIST — the `Ruled` arm of [`Self::prepare_rail_mark`].
    ///
    /// A filled band is the one thing this style refuses, so the rail cannot
    /// take the world's selected-row band the way every other style's rail does;
    /// it takes the world's own composition instead, through the SAME owner the
    /// rows beside it come out of ([`super::overlay_rules::rules_ink`]). The two
    /// regions then share one rhythm: both lists sit on the same row pitch from
    /// the same `first_top`, so the rail's rules and the pane's are the same
    /// rules, and the rail simply runs out where its categories do.
    ///
    /// The rail's own two spans mirror the pane's exactly. A hairline runs the
    /// LABEL measure — the rail column less the `2 * hpad` its measurement
    /// reserves — and the selection's heavy rule runs the full column, which is
    /// the very rect the filled band occupied. Same reach, different substance.
    /// THE ONE PLACE the recorded rail becomes rules — `(hairlines, selection)`.
    /// The draw path below and the law probe both come through here, so a law
    /// cannot grade a shape this frame would not have drawn.
    pub(in crate::render) fn workspace_rail_rule_ink(
        &self,
        mark: theme::RuleSelection,
    ) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
        let (hair, heavy) = self.rule_weights();
        let rows: Vec<super::overlay_rules::RuleRow> = self
            .workspace_rail_rows
            .iter()
            .map(
                |&([_, top, _, h], selected)| super::overlay_rules::RuleRow {
                    top,
                    bottom: top + h,
                    selected,
                },
            )
            .collect();
        // The rail's own column, read off the rail the shaper recorded: every
        // entry rect spans the full box, so the box IS any row's `[x, w]`.
        match self.workspace_rail_rows.first() {
            None => (Vec::new(), Vec::new()),
            Some(&([x, _, w, _], _)) => {
                let hpad = self.overlay_text_hpad();
                super::overlay_rules::rules_ink(
                    &rows,
                    mark,
                    &super::overlay_rules::RuleSpans {
                        hair,
                        heavy,
                        measure: (x, w - 2.0 * hpad),
                        band: (x, w),
                        mark: (
                            self.metrics.px(super::overlay_rules::RULE_MARK_LEN),
                            self.metrics.px(super::overlay_rules::RULE_MARK_GAP),
                        ),
                    },
                )
            }
        }
    }

    fn prepare_rail_rules(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        mark: theme::RuleSelection,
    ) {
        let (hairlines, selected) = self.workspace_rail_rule_ink(mark);
        // A rule is a drawn line, not a rounded surface — square ends, no
        // stroke, on both pipelines, which are shared with the chip skins and
        // would otherwise carry a corner across a world switch.
        self.overlay_lens_underline
            .set_color(self.rule_mark_ink(geom.rows_focused));
        self.overlay_lens_underline.set_corner(0.0);
        self.overlay_lens_underline.set_stroke(0.0);
        self.overlay_facet_ghost
            .set_color(theme::faint().rgba_bytes());
        self.overlay_facet_ghost.set_corner(0.0);
        self.overlay_facet_ghost.set_stroke(0.0);
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &selected);
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &hairlines);
    }

    /// The rail's ACTIVE entry's rect this frame — derived from the recorded
    /// rail rather than stored beside it, so the mark can never name a row the
    /// rail does not have.
    pub(in crate::render) fn workspace_rail_mark(&self) -> Option<[f32; 4]> {
        self.workspace_rail_rows
            .iter()
            .find(|(_, active)| *active)
            .map(|&(r, _)| r)
    }

    #[cfg(test)]
    pub(in crate::render) fn workspace_rail_mark_probe(&self) -> Option<[f32; 4]> {
        self.workspace_rail_mark()
    }

    /// TEST-ONLY: every rail entry's rect, active flag included.
    #[cfg(test)]
    pub(in crate::render) fn workspace_rail_rows_probe(&self) -> Vec<([f32; 4], bool)> {
        self.workspace_rail_rows.clone()
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
        let bottom = geom.card_y + geom.card_h - self.metrics.px(WORKSPACE_PAD);
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

    /// SHAPE the rail's labels into their own buffer and record EVERY entry's
    /// rect. Returns whether a rail was shaped at all.
    ///
    /// The active entry takes the SELECTED-BAND ink and the rest muted, the same
    /// figure/ground grammar every picker row uses; the rects are handed to
    /// [`super::overlay_rows`]'s facet-mark owner rather than drawn here, so the
    /// rail's mark and the content pane's come out of the same treatment.
    pub(in crate::render) fn workspace_shape_rail(
        &mut self,
        geom: &OverlayGeom,
        plan: &OverlayRowPlan,
    ) -> bool {
        self.workspace_rail_rows.clear();
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
        // THE ACTIVE ENTRY'S INK COMES FROM THE SAME PAIR ITS BAND DOES.
        // `prepare_rail_mark` lays a filled plate under this label, so the label
        // is drawn ON that fill and has to be chosen for it — `base_content` is
        // the right answer only on a world whose band already reads against it,
        // and on a world whose band IS `base_content` (an inverse-video
        // treatment) it is the fill's own colour, i.e. no label at all.
        let active = super::overlay_selected_label_ink();
        let muted = theme::muted().to_glyphon();
        let base = panel_attrs();
        let faint = theme::faint().to_glyphon();
        let mut spans: Vec<(String, glyphon::Color)> = Vec::new();
        let rows: Vec<([f32; 4], bool)> = (0..self.overlay_lens.len())
            .filter_map(|i| {
                let rect = self.workspace_rail_rect(geom, plan, i)?;
                Some((rect, self.overlay_lens[i].1))
            })
            .collect();
        for (i, (label, is_active)) in self.overlay_lens.iter().enumerate() {
            let line = match i {
                0 => label.clone(),
                _ => format!("\n{label}"),
            };
            spans.push((line, if *is_active { active } else { muted }));
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
        self.workspace_rail_rows = rows;
        true
    }
}
