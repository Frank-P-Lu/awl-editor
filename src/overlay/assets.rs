//! Asset-cleaner overlay construction and row retirement.

use super::{OverlayKind, OverlayState};

impl OverlayState {
    pub fn new_assets(orphans: Vec<crate::assets::Orphan>) -> Self {
        let n = orphans.len();
        let mut corpus = Vec::with_capacity(n);
        let mut secondary = Vec::with_capacity(n);
        for orphan in &orphans {
            secondary.push(crate::assets::secondary_label(orphan));
            corpus.push(orphan.rel.clone());
        }
        let mut state = Self::new_marked(
            OverlayKind::Assets,
            corpus,
            vec![false; n],
            vec![false; n],
            Vec::new(),
            Vec::new(),
            None,
        );
        state.set_secondaries(secondary);
        state
    }

    pub fn remove_asset_row(&mut self, rel: &str) -> bool {
        let Some(index) = self.rows.iter().position(|row| row.accept == rel) else {
            return false;
        };
        self.rows.remove(index);
        self.refilter();
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        true
    }
}
