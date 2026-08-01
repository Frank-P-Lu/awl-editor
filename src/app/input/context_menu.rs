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
        let (px, py) = self.cursor_px;
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
                has_selection: self.active.buffer.has_selection(),
                link: false,
                heading: false,
                heading_folded: false,
                misspelled: false,
                named_file: self.active.buffer.path().is_some(),
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
        self.active.buffer.seal_undo_group();
        let idx = self.hit_test_char();
        self.dragging = false;
        let selection_contains = self
            .active
            .buffer
            .selection_range()
            .is_some_and(|(start, end)| idx >= start && idx <= end);
        if !selection_contains {
            self.active.buffer.set_cursor(idx);
            self.active.buffer.clear_mark();
            self.active.buffer.set_anchor(idx);
        }
        self.active.extra.shift_selecting = false;
        let (line, col) = self.hit_test_line_col();
        let byte = self.active.buffer.char_to_byte(idx);
        let text = self.active.buffer.text();
        let misspelled = self.spell.as_ref().is_some_and(|sc| {
            sc.suggest_at(&text, line, col, self.active.buffer.syntax_lang())
                .is_some()
        });
        if misspelled {
            self.active.buffer.set_cursor(idx);
            self.active.buffer.clear_mark();
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
                heading: self.active.buffer.is_markdown()
                    && crate::markdown::headings(&text)
                        .iter()
                        .any(|h| h.line == line),
                heading_folded: self.active.buffer.folds().contains(&line),
                misspelled: false,
                named_file: self.active.buffer.path().is_some(),
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
        let idx = self.active.buffer.line_col_to_char(line, 0);
        self.active.buffer.set_cursor(idx);
        self.active.buffer.clear_mark();
        let state = crate::context_menu::ContextState {
            has_selection: false,
            link: false,
            heading: true,
            heading_folded: self.active.buffer.folds().contains(&line),
            misspelled: false,
            named_file: self.active.buffer.path().is_some(),
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
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }
}
