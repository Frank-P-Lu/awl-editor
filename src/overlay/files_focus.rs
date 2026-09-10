//! Keyboard-focus routing within the dedicated Files surface.

use super::{FilesFocus, OverlayKind, OverlayState, RowMeta, RowMetaTag};

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
        if self.items.iter().any(|&index| self.files_choice(index)) {
            route.push(Choices);
        }
        if self.has_files_meta(RowMetaTag::FolderChooser) {
            route.push(ChangeFolder);
        }
        if self.has_files_meta(RowMetaTag::NewDocument) {
            route.push(NewDocument);
        }
        let at = route
            .iter()
            .position(|focus| *focus == self.files_focus)
            .unwrap_or(0) as isize;
        let next = (at + delta).rem_euclid(route.len() as isize) as usize;
        self.files_focus = route[next];
        match self.files_focus {
            ChangeFolder => self.files_select_meta(RowMetaTag::FolderChooser),
            NewDocument => self.files_select_meta(RowMetaTag::NewDocument),
            Choices => self.files_select_first_choice(),
            Query | Files | Recent | Up => {}
        }
    }

    pub fn files_select_choices(&mut self) {
        if self.kind != OverlayKind::Goto || self.goto_outline_only {
            return;
        }
        self.files_focus = FilesFocus::Choices;
        self.files_select_first_choice();
    }

    pub fn files_rows_focused(&self) -> bool {
        !self.files_mode
            || matches!(
                self.files_focus,
                FilesFocus::Choices | FilesFocus::ChangeFolder | FilesFocus::NewDocument
            )
    }

    fn files_choice(&self, index: usize) -> bool {
        !matches!(
            self.rows[index].meta,
            RowMeta::FolderChooser | RowMeta::NewDocument
        )
    }

    fn has_files_meta(&self, tag: RowMetaTag) -> bool {
        self.items
            .iter()
            .any(|&index| self.rows[index].meta.tag() == tag)
    }

    fn files_select_first_choice(&mut self) {
        if self
            .selected_corpus_index()
            .is_some_and(|index| self.files_choice(index))
        {
            return;
        }
        if let Some(position) = self
            .items
            .iter()
            .position(|&index| self.files_choice(index))
        {
            self.selected = position;
            self.scroll_to_selected();
        }
    }

    fn files_select_meta(&mut self, tag: RowMetaTag) {
        if let Some(position) = self
            .items
            .iter()
            .position(|&index| self.rows[index].meta.tag() == tag)
        {
            self.selected = position;
            self.scroll_to_selected();
        }
    }
}
