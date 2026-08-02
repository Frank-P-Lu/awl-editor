use crate::app::*;

impl App {
    pub(in crate::app) fn on_right_press(
        &mut self,
        exit: &dyn schedule::Exit,
        over_writing_column: bool,
    ) {
        if self.workspace_state.overlay_open() {
            let _ = self.apply(Action::Cancel, false, exit, crate::stats::Door::Chord);
        }
        let (px, py) = self.input.pointer.cursor_px;
        if self.summon_heading_context(px, py) {
            return self.finish_context_summon();
        }
        let target = self.gpu.as_ref().and_then(|gpu| {
            gpu.pipeline
                .page_resize_edge_at(px)
                .map(|edge| match edge {
                    crate::render::ResizeEdge::Left => crate::context_menu::ContextTarget::LeftEdge,
                    crate::render::ResizeEdge::Right => {
                        crate::context_menu::ContextTarget::RightEdge
                    }
                })
                .or_else(|| {
                    gpu.pipeline
                        .gutter_context_target(px, py, gpu.config.height)
                })
        });
        if let Some(target) = target {
            let state = crate::context_menu::ContextState {
                has_selection: self.document.buffer().has_selection(),
                link: false,
                heading: false,
                heading_folded: false,
                misspelled: false,
                named_file: self.document.buffer().path().is_some(),
            };
            let rows =
                crate::context_menu::rows(target, state, crate::commands::Platform::current());
            if !rows.is_empty() {
                self.workspace_state
                    .summon_context(crate::context_menu::overlay(rows, (px, py)));
            }
            return self.finish_context_summon();
        }
        if !over_writing_column {
            return;
        }
        self.document.seal_undo_group();
        let idx = self.hit_test_char();
        self.input.pointer.dragging = false;
        let selection_contains = self
            .document
            .buffer()
            .selection_range()
            .is_some_and(|(start, end)| idx >= start && idx <= end);
        if !selection_contains {
            self.document.set_cursor(idx);
            self.document.clear_mark();
            self.document.set_anchor(idx);
        }
        self.document.set_shift_selecting(false);
        let (line, col) = self.hit_test_line_col();
        let byte = self.document.buffer().char_to_byte(idx);
        let text = self.document.buffer().text();
        let misspelled = self.document.spell_suggestion_target(line, col).is_some();
        if misspelled {
            self.document.set_cursor(idx);
            self.document.clear_mark();
            let _ = self.apply(
                Action::OpenSpellSuggest,
                false,
                exit,
                crate::stats::Door::Chord,
            );
        } else {
            let state = crate::context_menu::ContextState {
                has_selection: selection_contains,
                link: crate::markdown::link_at(&text, byte).is_some(),
                heading: self.document.buffer().is_markdown()
                    && crate::markdown::headings(&text)
                        .iter()
                        .any(|h| h.line == line),
                heading_folded: self.document.buffer().folds().contains(&line),
                misspelled: false,
                named_file: self.document.buffer().path().is_some(),
            };
            let rows = crate::context_menu::rows(
                crate::context_menu::document_target(state),
                state,
                crate::commands::Platform::current(),
            );
            self.workspace_state
                .summon_context(crate::context_menu::overlay(rows, (px, py)));
        }
        self.finish_context_summon();
    }

    fn summon_heading_context(&mut self, px: f32, py: f32) -> bool {
        let Some(line) = self.fold_chevron_at_pointer() else {
            return false;
        };
        let idx = self.document.buffer().line_col_to_char(line, 0);
        self.document.set_cursor(idx);
        self.document.clear_mark();
        let state = crate::context_menu::ContextState {
            has_selection: false,
            link: false,
            heading: true,
            heading_folded: self.document.buffer().folds().contains(&line),
            misspelled: false,
            named_file: self.document.buffer().path().is_some(),
        };
        let rows = crate::context_menu::rows(
            crate::context_menu::ContextTarget::Heading,
            state,
            crate::commands::Platform::current(),
        );
        self.workspace_state
            .summon_context(crate::context_menu::overlay(rows, (px, py)));
        true
    }

    fn finish_context_summon(&mut self) {
        self.sync_view(true);
        self.request_frame();
    }
}
