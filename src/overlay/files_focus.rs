//! Keyboard-focus routing within the dedicated Files surface.

use super::{FilesFocus, OverlayKind, OverlayState};

impl OverlayState {
    pub fn files_focus_step(&mut self, delta: isize) {
        use FilesFocus::*;
        if self.kind != OverlayKind::Goto || self.goto_outline_only {
            return;
        }
        let mut route = vec![Query, Files, Recent];
        if self.facet_lens == 0 && self.browse_dir.is_some() {
            route.push(Up);
        }
        if !self.items.is_empty() {
            route.push(Choices);
        }
        route.push(ChangeFolder);
        route.push(NewDocument);
        let at = route
            .iter()
            .position(|focus| *focus == self.files_focus)
            .unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(route.len() as isize) as usize;
        self.files_focus = route[next];
        if self.files_focus == Choices {
            self.files_select_choices();
        }
    }

    pub fn files_select_choices(&mut self) {
        if self.kind != OverlayKind::Goto || self.goto_outline_only {
            return;
        }
        self.files_focus = FilesFocus::Choices;
        self.files_select_first_choice();
        self.scroll_to_selected();
    }

    pub fn files_rows_focused(&self) -> bool {
        !self.files_mode || self.files_focus == FilesFocus::Choices
    }

    fn files_select_first_choice(&mut self) {
        if self.selected_corpus_index().is_some() {
            return;
        }
        if !self.items.is_empty() {
            self.selected = 0;
            self.scroll_to_selected();
        }
    }
}
