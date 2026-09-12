//! Pointer navigation and query dragging within summoned pickers.

use crate::app::*;

impl App {
    pub(in crate::app) fn overlay_hover(&mut self) {
        let (px, py) = self.input.pointer.cursor_px;
        let Some(gpu) = self.frame.gpu() else { return };

        // Grid hover shares click geometry and the real-motion gate. It must
        // precede row handling because this picker has no candidate rows.
        if self
            .workspace_state
            .overlay()
            .is_some_and(|ov| ov.table_dims.is_some())
        {
            let hit = gpu.pipeline.table_dims_cell_at(px, py);
            let Some(ov) = self.workspace_state.overlay_mut() else {
                return;
            };
            if !ov.table_dims_hover_at(px, py, hit) {
                return;
            }
            self.sync_view(false);
            self.request_frame();
            return;
        }

        // The real-motion gate rejects reflow and duplicate pointer reports;
        // hovering a list edge never moves its scroll window.
        let kind = match self.workspace_state.overlay_mut() {
            Some(ov) => {
                if !gpu.pipeline.resolve_overlay_hover(ov, px, py) {
                    return;
                }
                ov.kind
            }
            None => return,
        };
        // Record only hover that passed the movement gate and changed selection.
        #[cfg(not(target_arch = "wasm32"))]
        if crate::probe::recording() {
            let sel = self.overlay_selection_probe();
            crate::probe::trace(format_args!(
                "hover_took_selection at=({px:.1},{py:.1}) {sel:?}"
            ));
        }
        let prev = crate::theme::active();
        if let Some(ov) = self.workspace_state.overlay() {
            // Hover previews without re-anchoring the card.
            crate::actions::preview_overlay(ov);
        }
        // Apply colors now; defer font reshaping until the preview settles.
        if kind == crate::overlay::OverlayKind::Theme {
            self.retint_theme_preview(prev);
        }
        self.sync_view(false);
        self.request_frame();
    }

    pub(in crate::app) fn overlay_wheel(&mut self, lines: f32) {
        let delta = -(lines.round() as isize); // wheel DOWN (lines < 0) advances (↓)
        if delta == 0 {
            return;
        }
        let kind = match self.workspace_state.overlay_mut() {
            Some(ov) => {
                ov.move_sel(delta);
                ov.kind
            }
            None => return,
        };
        let prev = crate::theme::active();
        if let Some(ov) = self.workspace_state.overlay_mut() {
            crate::actions::preview_move(ov);
        }
        // Wheel navigation bypasses App::apply: stamp the resting pointer so a
        // duplicate CursorMoved cannot take selection back after scrolling.
        let (px, py) = self.input.resting_pointer().px();
        if let Some(ov) = self.workspace_state.overlay_mut() {
            ov.arm_hover_baseline(px, py);
        }
        if kind == crate::overlay::OverlayKind::Theme {
            self.retint_theme_preview(prev);
        }
        self.sync_view(false);
        self.request_frame();
    }

    /// Scrub the query caret while the pointer remains over the field. Leaving
    /// its hit region pauses the drag rather than clamping to an unrelated glyph.
    pub(in crate::app) fn on_query_drag(&mut self) {
        let (px, py) = self.input.pointer.cursor_px;
        let Some(idx) = self
            .frame
            .gpu()
            .and_then(|g| g.pipeline.overlay_query_char_at(px, py))
        else {
            return;
        };
        if let Some(ov) = self.workspace_state.overlay_mut() {
            ov.query_set_caret(idx);
        }
        self.workspace_state
            .focus_settings_if_open(crate::overlay::workspace::SettingsFocus::Search);
        self.sync_view(true);
        self.request_frame();
    }

    /// A rail press selects its category and enters that category's content.
    /// Settings names Controls precisely; other workspaces keep their coarse
    /// detail-stage transition.
    fn focus_workspace_rail_at(&mut self, rail_idx: usize) {
        if let Some(overlay) = self.workspace_state.overlay_mut() {
            overlay.set_facet_lens(rail_idx);
        }
        if !self
            .workspace_state
            .focus_settings_if_open(crate::overlay::workspace::SettingsFocus::Controls)
        {
            self.workspace_state.focus_workspace_detail();
        }
        self.sync_view(true);
        self.request_frame();
    }

    /// Use press-time hit tests, never cached hover or release position. Rows
    /// accept through the shared action path, except range labels select only.
    /// Query presses place their caret and arm dragging. Interior gaps consume the
    /// press; click-away cancels through the shared path so previews revert.
    pub(in crate::app) fn overlay_click(&mut self, exit: &dyn schedule::Exit) {
        let (px, py) = self.input.pointer.cursor_px;
        let (row_hit, lens_hit, rail_hit, query_hit, files_hit, card, table_dims_hit) = self
            .frame
            .gpu()
            .map(|g| {
                (
                    g.pipeline.overlay_row_at(px, py),
                    g.pipeline.overlay_lens_at(px, py),
                    g.pipeline.workspace_rail_at(px, py),
                    g.pipeline.overlay_query_char_at(px, py),
                    g.pipeline.files_surface_action_at(px, py),
                    g.pipeline.overlay_card_rect(),
                    g.pipeline.table_dims_cell_at(px, py),
                )
            })
            .unwrap_or((None, None, None, None, None, None, None));

        // Grid cells commit through the same action as Enter.
        if let Some((row, col)) = table_dims_hit {
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.table_dims_pick(row, col);
            }
            self.apply(Action::Newline, false, exit, crate::stats::Door::Chord);
            self.sync_view(true);
            self.request_frame();
            return;
        }
        // Rail clicks use the same lens and focus transitions as keyboard entry.
        if let Some(rail_idx) = rail_hit {
            self.focus_workspace_rail_at(rail_idx);
            return;
        }

        if let Some(action) = files_hit {
            if let Some(overlay) = self.workspace_state.overlay_mut() {
                overlay.files_focus = match action {
                    crate::render::FilesSurfaceAction::Up => crate::overlay::FilesFocus::Up,
                    crate::render::FilesSurfaceAction::ChangeFolder => {
                        crate::overlay::FilesFocus::ChangeFolder
                    }
                    crate::render::FilesSurfaceAction::NewDocument => {
                        crate::overlay::FilesFocus::NewDocument
                    }
                };
            }
            self.apply(Action::Newline, false, exit, crate::stats::Door::Chord);
            self.sync_view(true);
            self.request_frame();
            return;
        }

        // FACETED PICKER: a click on a LENS label switches the facet (keeping the
        // selection), then previews + re-tints — the pointing counterpart to LEFT/RIGHT.
        // Handled before the row hit-test (the strip sits above the rows, never overlaps).
        if let Some(lens_idx) = lens_hit {
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.set_facet_lens(lens_idx);
            }
            let prev = crate::theme::active();
            if let Some(ov) = self.workspace_state.overlay() {
                crate::actions::preview_overlay(ov);
            }
            self.retint_theme_preview(prev);
            self.sync_view(false);
            self.request_frame();
            return;
        }

        if self.begin_range_drag() {
            self.sync_view(true);
            self.request_frame();
            return;
        }

        if let Some(idx) = row_hit {
            // ON a row: ACCEPT through the shared apply path — byte-for-byte the same
            // as Enter on the highlighted row (open / run / commit / descend / replace).
            if let Some(ov) = self.workspace_state.overlay_mut()
                && idx < ov.items.len()
            {
                ov.selected = idx;
            }
            self.workspace_state
                .focus_settings_if_open(crate::overlay::workspace::SettingsFocus::Controls);
            // Range labels select only; Enter would open the numeric editor.
            let is_range = self
                .workspace_state
                .overlay()
                .is_some_and(|ov| ov.range_of_item(idx).is_some());
            if is_range {
                self.sync_view(true);
                self.request_frame();
                return;
            }
            self.apply(Action::Newline, false, exit, crate::stats::Door::Chord);
        } else if let Some(char_idx) = query_hit {
            // ON the query line, off a row: place the field's own caret there
            // and arm the drag so a press-drag scrubs it, rather than falling
            // through to the generic "inside, off a row" swallow below.
            if let Some(ov) = self.workspace_state.overlay_mut() {
                ov.query_set_caret(char_idx);
            }
            self.workspace_state
                .focus_settings_if_open(crate::overlay::workspace::SettingsFocus::Search);
            self.input.pointer.query_drag = true;
        } else {
            let inside = card
                .map(|[x, y, w, h]| px >= x && px <= x + w && py >= y && py <= y + h)
                .unwrap_or(false);
            if inside {
                return;
            }
            self.apply(Action::Cancel, false, exit, crate::stats::Door::Chord);
        }
        self.sync_view(true);
        self.request_frame();
    }
}
