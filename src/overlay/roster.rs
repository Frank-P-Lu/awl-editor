//! Stable display rosters for overlay rows and faceted hug measurement.

use super::{HugRoster, OverlayState, RowMeta};
use std::sync::Arc;

impl OverlayState {
    pub(super) fn refresh_hug_roster(&mut self) {
        if !self.is_faceting() {
            self.hug_roster = None;
            return;
        }
        self.hug_roster = Some(Arc::new(HugRoster {
            primary: self.hug_primary_strings(),
            secondary: self.hug_secondary_strings(),
            candidate_rows: self.rows.len(),
        }));
    }

    /// Immutable summon-time corpus for a faceted picker. A view push clones
    /// only this `Arc`; mutations refresh it at their own seam.
    pub fn hug_roster(&self) -> Option<Arc<HugRoster>> {
        self.hug_roster.clone()
    }

    fn display_of(&self, i: usize) -> String {
        super::build::row_display(self.kind, &self.rows[i], self.browse_dir.as_deref())
    }

    pub fn item_strings(&self) -> Vec<String> {
        self.items.iter().map(|&i| self.display_of(i)).collect()
    }

    /// The full summon corpus keeps a right-anchored faceted card stationary
    /// while a lens or fuzzy query changes the visible projection.
    pub fn hug_primary_strings(&self) -> Vec<String> {
        let Some(scheme) = self.facet_scheme() else {
            return Vec::new();
        };
        let mut text: Vec<String> = (0..self.rows.len()).map(|i| self.display_of(i)).collect();
        text.push(self.title());
        text.extend(scheme.strip.iter().map(|facet| facet.label.to_string()));
        text.push("no matches".to_string());
        text.push(self.kind.empty_corpus_message().to_string());
        text.extend(
            scheme
                .strip
                .iter()
                .filter_map(|facet| self.kind.empty_lens_message(facet.id))
                .map(str::to_string),
        );
        text
    }

    /// The full secondary column in renderer priority order: authored labels,
    /// then Go-to edit times, then git tags.
    pub fn hug_secondary_strings(&self) -> Vec<String> {
        if !self.is_faceting() {
            return Vec::new();
        }
        let authored: Vec<String> = self.rows.iter().map(|row| row.secondary.clone()).collect();
        if authored.iter().any(|label| !label.is_empty()) {
            return authored;
        }
        let times: Vec<String> = self
            .rows
            .iter()
            .map(|row| match &row.meta {
                RowMeta::GotoFile { time } => time.clone(),
                _ => String::new(),
            })
            .collect();
        if times.iter().any(|label| !label.is_empty()) {
            return times;
        }
        if self.rows.iter().any(|row| row.git) {
            return self
                .rows
                .iter()
                .map(|row| if row.git { "git".into() } else { String::new() })
                .collect();
        }
        Vec::new()
    }
}
