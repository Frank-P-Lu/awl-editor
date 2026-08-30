use super::*;

impl App {
    /// Project the honest no-document state. The start surface is canvas chrome,
    /// not a synthetic buffer: there is no caret, page, text, filesystem claim,
    /// or document metadata hiding behind its two actions. A summoned Go-to card
    /// still uses the ordinary overlay projection over that surface.
    pub(super) fn sync_zero_document_view(&mut self) {
        if self.persistence.title_cache_stale(false) {
            self.update_title();
        }
        let ov = self.workspace_state.overlay();
        let mut view = ViewState::base();
        view.document_active = false;
        view.zoom = self.frame.zoom();
        view.overlay_active = ov.is_some();
        view.overlay_align = ov.map(|o| o.align);
        view.overlay_crisp = ov.is_some_and(|o| o.kind.keeps_backdrop_crisp());
        view.overlay_query = ov.map(|o| o.query.text().to_string()).unwrap_or_default();
        view.overlay_query_caret = ov.map(|o| o.query.caret()).unwrap_or(0);
        view.overlay_query_selection = ov.and_then(|o| o.query.selection_range());
        view.overlay_title = ov
            .filter(|o| o.kind.draws_title_prefix())
            .map(|o| o.kind.title().to_string())
            .unwrap_or_default();
        view.overlay_row_path_splits = ov.map(|o| o.kind.row_path_splits()).unwrap_or(false);
        view.overlay_items = ov.map(|o| o.item_strings()).unwrap_or_default();
        view.overlay_hug_roster = ov.and_then(crate::overlay::OverlayState::hug_roster);
        view.overlay_empty = ov.and_then(|o| o.empty_notice());
        view.overlay_bindings = ov.map(|o| o.item_bindings()).unwrap_or_default();
        view.overlay_ranges = ov.map(|o| o.item_range_fracs()).unwrap_or_default();
        view.overlay_times = ov.map(|o| o.item_times()).unwrap_or_default();
        view.overlay_git = ov.map(|o| o.item_git_tags()).unwrap_or_default();
        view.overlay_selected = ov.map(|o| o.selected).unwrap_or(0);
        view.overlay_scroll = ov.map(|o| o.scroll).unwrap_or(0);
        view.overlay_window_rows = ov.map(|o| o.window_rows()).unwrap_or(12);
        view.overlay_hint = self.workspace_state.journey().foot_hint();
        view.overlay_lens = ov.map(|o| o.lens_strip()).unwrap_or_default();
        view.overlay_workspace = ov.is_some_and(|o| o.workspace_shape().is_some());
        view.overlay_rows_primary = ov.is_some_and(|o| {
            o.workspace_shape()
                .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary)
        });
        view.overlay_sections = ov.map(|o| o.item_sections()).unwrap_or_default();
        view.overlay_location = ov
            .and_then(|o| o.location())
            .map(std::string::ToString::to_string);
        view.config_keys = self.config.keys.clone();
        view.config_linux_keep = self.config.effective_linux_keep();
        view.config_keymap_flavor = self.config.keymap_flavor();
        view.notice = self.frame.notice().owned().unwrap_or_default();
        view.notice_kind = self.frame.notice().kind();
        view.cjk_priority = self.config.cjk_priority_or_default();
        self.frame.gpu_mut().unwrap().pipeline.set_view(&view);
        let _ = self.frame.take_caret_motion_flags();
    }
}
