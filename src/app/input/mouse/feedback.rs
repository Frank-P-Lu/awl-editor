//! Cursor shape and fold-hover feedback from the existing hit tests.

use crate::app::*;

impl App {
    /// Resolve the OS cursor from existing hit tests, through pointer_sync.
    /// Skip OS writes and cache updates while hidden; the next pointer move compares
    /// the fresh context against the last visible cursor.
    pub(in crate::app::input) fn sync_cursor_icon(&mut self) {
        let Some(gpu) = self.frame.gpu() else { return };
        let (px, py) = self.input.pointer.cursor_px;
        // All affordances reuse the hit tests that their press paths consume.
        let overlay_open = self.workspace_state.overlay_open();
        let over_clickable_overlay_row =
            overlay_open && gpu.pipeline.overlay_row_at(px, py).is_some();
        let over_clickable_lens = overlay_open
            && (gpu.pipeline.overlay_lens_at(px, py).is_some()
                || gpu.pipeline.workspace_rail_at(px, py).is_some()
                || gpu.pipeline.files_surface_action_at(px, py).is_some()
                || gpu.pipeline.theme_panel_action_at(px, py).is_some());
        let over_table_dims_cell =
            overlay_open && gpu.pipeline.table_dims_cell_at(px, py).is_some();
        let over_query_input = overlay_open && gpu.pipeline.over_overlay_query(px, py);
        let over_outline_row = !overlay_open
            && gpu
                .pipeline
                .outline_hit_line(px, py, gpu.config.height)
                .is_some();
        let over_stack_row = !overlay_open
            && (gpu
                .pipeline
                .gutter_stack_hit(px, py, gpu.config.height)
                .is_some()
                || gpu.pipeline.start_action_at(px, py).is_some());
        let image_hover = gpu
            .pipeline
            .image_handle_at(px, py)
            .map(|(_, handle, _)| handle);
        let over_menu_hand = gpu.pipeline.menubar_hand_at(px, py);
        let over_menu_bar = gpu.pipeline.over_menu_surface(px, py);
        let panel_hit = (!overlay_open)
            .then(|| gpu.pipeline.panel_hit(px, py))
            .flatten();
        let over_case_toggle = matches!(panel_hit, Some(crate::render::PanelHit::CaseToggle));
        let over_panel_button = matches!(
            panel_hit,
            Some(crate::render::PanelHit::Close)
                | Some(crate::render::PanelHit::RevealReplace)
                | Some(crate::render::PanelHit::NavPrev)
                | Some(crate::render::PanelHit::NavNext)
                | Some(crate::render::PanelHit::ReplaceButton)
                | Some(crate::render::PanelHit::ReplaceAllButton)
        );
        let over_panel_field = matches!(
            panel_hit,
            Some(crate::render::PanelHit::Find) | Some(crate::render::PanelHit::Replace)
        );
        let over_panel = matches!(panel_hit, Some(crate::render::PanelHit::Elsewhere));
        // The RAW summon bit, deliberately ladder-free — see its own doc.
        let summoned = self.workspace_state.popover_summon_bit();
        let over_popover_button = summoned && gpu.pipeline.popover_hit(px, py).is_some();
        let over_fold_chevron = !overlay_open && gpu.pipeline.fold_chevron_hit(px, py).is_some();
        // The hand appears exactly when a press WOULD follow: the same gesture
        // predicate the press path asks (`keymap::follows_link`).
        let over_modified_link = self.document.has_active()
            && !overlay_open
            && gpu.pipeline.over_writing_column(px)
            && crate::context_menu::modified_link_hover(
                crate::keymap::follows_link(
                    crate::convention::Convention::current(),
                    &self.config.follow,
                    crate::keymap::PointerButton::Primary,
                    self.input.keyboard.mods.state(),
                ),
                self.followable_at_pointer(),
            );
        let ctx = crate::cursor_shape::CursorContext {
            dragging_edge: self.input.pointer.page_resizing,
            dragging_text: self.input.pointer.dragging,
            overlay_open,
            over_edge: self.document.has_active() && gpu.pipeline.page_resize_hover(px),
            over_text: self.document.has_active() && gpu.pipeline.over_writing_column(px),
            over_clickable_overlay_row,
            over_clickable_lens,
            over_table_dims_cell,
            over_query_input,
            over_outline_row: over_outline_row || over_modified_link,
            over_stack_row,
            over_menu_hand,
            over_menu_bar,
            over_case_toggle,
            over_panel_button,
            over_panel_field,
            over_panel,
            image_drag: self.input.pointer.image_resizing.map(|d| d.handle),
            image_hover,
            over_popover_button,
            over_fold_chevron,
        };
        let desired = crate::cursor_shape::cursor_icon_for(ctx);
        let hidden = self.input.pointer.pointer_hide == crate::pointer_hide::PointerHide::Hidden;
        if let Some(icon) =
            crate::cursor_shape::cursor_icon_change(self.input.pointer.cursor_icon, desired, hidden)
        {
            gpu.window.set_cursor(icon);
            self.input.pointer.cursor_icon = icon;
        }
    }

    /// Mirror the filtered Markdown row under the pointer, including expanded
    /// headings. Redraw only when it changes; pointer loss clears it separately
    /// through `clear_pointer_hover_state`.
    pub(in crate::app::input) fn update_fold_hover(&mut self) {
        if !self.document.has_active() {
            if self
                .frame
                .gpu_mut()
                .is_some_and(|gpu| gpu.pipeline.set_hover_line(None))
            {
                self.request_frame();
            }
            return;
        }
        let over_col = self.document.buffer().is_markdown() && self.pointer_over_writing_column();
        let (px, py) = self.input.pointer.cursor_px;
        let scroll = self.document.scroll();
        let Some(gpu) = self.frame.gpu_mut() else {
            return;
        };
        let line = if over_col {
            Some(gpu.pipeline.hit_test_scroll(px, py, scroll).0)
        } else {
            None
        };
        if gpu.pipeline.set_hover_line(line) {
            self.request_frame();
        }
    }
}
