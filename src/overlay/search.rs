//! "Search in folder…"'s own `refilter` branch: re-MATCH the already-loaded
//! corpus (`OverlayState::search_corpus`, loaded once at summon) against the
//! typed query on every keystroke, off the frame path and against
//! [`crate::search_folder::SearchBudget`]'s bound — never a fresh disk walk.

use super::{OverlayRow, OverlayState, RowMeta};
use crate::search_folder::{self, SearchBudget};

impl OverlayState {
    pub(super) fn rebuild_search_rows(&mut self) {
        let hits = search_folder::search(
            &self.search_corpus,
            self.query.text(),
            &SearchBudget::default(),
        );
        self.rows = hits
            .into_iter()
            .map(|hit| OverlayRow {
                accept: hit.snippet,
                // 1-based for the reader (`path:line` reads like every other
                // line-addressed tool); `RowMeta::SearchHit::line` beside it
                // stays 0-based, `line_col_to_char`'s own unit.
                secondary: format!("{}:{}", hit.path, hit.line + 1),
                is_dir: false,
                git: false,
                meta: RowMeta::SearchHit {
                    path: hit.path,
                    line: hit.line,
                    col: hit.col,
                    hl_start: hit.hl_start,
                    hl_end: hit.hl_end,
                },
                range: None,
            })
            .collect();
        self.items = (0..self.rows.len()).collect();
        self.item_sections = vec![String::new(); self.rows.len()];
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        self.scroll_to_selected();
        self.diff_scroll = 0;
        self.refresh_hug_roster();
    }
}
