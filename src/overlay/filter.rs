//! Ranking, visibility filtering, and faceted bucketing for picker rows.

use super::{OverlayKind, OverlayRow, OverlayState, RowMeta};
use crate::fuzzy::{self, Tier};

impl OverlayState {
    pub fn refilter(&mut self) {
        let accepts = self.accepts();
        let mut scored = fuzzy::rank(self.query.text(), &accepts, |i| {
            if self.open.contains(&i) {
                Tier::Open
            } else if self.recent.contains(&i) {
                Tier::Recent
            } else {
                Tier::Corpus
            }
        });
        // MRU is the tiebreak between fuzzy score and corpus order. Inert when
        // `recent` is empty, preserving the plain rank byte-for-byte.
        let recent_rank = |ci: usize| {
            self.recent
                .iter()
                .position(|&x| x == ci)
                .unwrap_or(usize::MAX)
        };
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| recent_rank(a.index).cmp(&recent_rank(b.index)))
                .then_with(|| a.index.cmp(&b.index))
        });
        let mut ranked: Vec<usize> = scored.into_iter().map(|r| r.index).collect();
        self.park_terminal_rows_last(&mut ranked);
        self.retain_visible_rows(&mut ranked);
        self.apply_active_facet(ranked);
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        self.scroll_to_selected();
        self.diff_scroll = 0;
    }

    /// Rows that act on something other than the query stay reachable last,
    /// even when their own label fuzzy-matches ahead of a real answer.
    fn park_terminal_rows_last(&self, ranked: &mut Vec<usize>) {
        if !self.rows.iter().any(|r| r.meta.terminal()) {
            return;
        }
        let terminal: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.meta.terminal())
            .map(|(ci, _)| ci)
            .collect();
        ranked.retain(|ci| !terminal.contains(ci));
        ranked.extend(terminal);
    }

    fn retain_visible_rows(&self, ranked: &mut Vec<usize>) {
        // The folder chooser is a fallback action belonging only to Folders.
        ranked.retain(|&i| {
            !matches!(self.rows[i].meta, RowMeta::FolderChooser)
                || self.active_facet_id() == Some("folders")
        });
        ranked.retain(|&i| {
            !matches!(
                self.rows.get(i).map(|r| &r.meta),
                Some(RowMeta::CommandHidden)
            )
        });
        if !crate::file_visibility::all_on() && self.kind.hides_dotfiles() {
            let exempt = |row: &OverlayRow| {
                row.accept == super::HERE_ACCEPT
                    || matches!(row.meta, RowMeta::GotoHeading { .. })
                    || (self.kind == OverlayKind::Project && super::is_remembered_root(&row.accept))
            };
            ranked.retain(|&i| {
                let row = &self.rows[i];
                exempt(row) || !crate::index::is_hidden_entry(&row.accept)
            });
            // Browse stamps unsupported files with a non-empty secondary label.
            if self.kind == OverlayKind::Browse {
                ranked.retain(|&i| self.rows[i].is_dir || self.rows[i].secondary.is_empty());
            }
        }
        if self.kind == OverlayKind::Goto
            && self.facet_lens != 0
            && self.active_facet_id() != Some("headings")
        {
            ranked.retain(|&i| !matches!(self.rows[i].meta, RowMeta::GotoHeading { .. }));
        }
    }

    fn apply_active_facet(&mut self, ranked: Vec<usize>) {
        let scheme = self.facet_scheme();
        let Some(sc) = scheme.filter(|_| self.filters_to_active_facet()) else {
            self.item_sections = vec![String::new(); ranked.len()];
            self.items = ranked;
            return;
        };
        let mut items = Vec::with_capacity(ranked.len());
        let mut sections = Vec::with_capacity(ranked.len());
        for sect in sc.strip[self.facet_lens].sections {
            for &ci in &ranked {
                let row = &self.rows[ci];
                let item = crate::facets::FacetItem {
                    accept: &row.accept,
                    is_dir: row.is_dir,
                    is_git: row.git,
                    recent: self.recent.contains(&ci),
                    heading: matches!(row.meta, RowMeta::GotoHeading { .. }),
                    ts: match row.meta {
                        RowMeta::History { ts, .. } => Some(ts),
                        _ => None,
                    },
                    now: self.facet_now,
                    session_start: self.facet_session_start,
                };
                if (sc.bucket)(item, self.facet_lens) == Some(*sect) {
                    items.push(ci);
                    sections.push((*sect).to_string());
                }
            }
        }
        self.items = items;
        self.item_sections = sections;
    }
}
