//! Ranking, visibility filtering, and faceted bucketing for picker rows.

use super::{OverlayKind, OverlayRow, OverlayState, RowMeta};
use crate::fuzzy::{self, Tier};

impl OverlayState {
    pub fn refilter(&mut self) {
        self.sync_goto_line_row();
        self.sync_move_new_folder_row();
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
        self.pin_move_here(&mut ranked);
        self.retain_visible_rows(&mut ranked);
        self.apply_active_facet(ranked);
        if self.selected >= self.items.len() {
            self.selected = self.items.len().saturating_sub(1);
        }
        self.scroll_to_selected();
        self.diff_scroll = 0;
    }

    /// The ONE-BASED destination line the CURRENT query names, clamped to
    /// `[1, goto_line_count]` -- `None` unless the trimmed query is a bare,
    /// non-empty digit string AND a buffer's line count is known
    /// (`goto_line_count > 0`, set once by `attach_line_jump`). CLAMPING
    /// wild input rather than refusing it matches the shared jump owner
    /// every other caller already gets for free (`Buffer::line_col_to_char`
    /// clamps both its line and column arguments); the row's LABEL is always
    /// written from this same clamped value (`sync_goto_line_row`), so what
    /// it promises is always the line Enter actually reaches, never a silent
    /// surprise past the buffer's end.
    pub(super) fn goto_line_target(&self) -> Option<usize> {
        if self.goto_line_count == 0 {
            return None;
        }
        let text = self.query.text();
        let trimmed = text.trim();
        if trimmed.is_empty() || !trimmed.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let typed: u64 = trimmed.parse().unwrap_or(u64::MAX);
        Some(typed.max(1).min(self.goto_line_count as u64) as usize)
    }

    /// Refresh Go to Line's ONE fixed row from the current query, before
    /// ranking runs (`refilter`'s first step) -- the row itself never moves
    /// or gets re-created (`attach_line_jump` appends it once); only its
    /// label and `RowMeta::GotoLine { line }` change per keystroke. Visibility
    /// (hiding the row entirely when the query names no valid line) is
    /// `retain_visible_rows`'s job, not this one.
    fn sync_goto_line_row(&mut self) {
        if self.kind != OverlayKind::Goto {
            return;
        }
        let Some(one_based) = self.goto_line_target() else {
            return;
        };
        if let Some(row) = self
            .rows
            .iter_mut()
            .find(|r| matches!(r.meta, RowMeta::GotoLine { .. }))
        {
            row.accept = format!("Go to line {one_based}");
            row.meta = RowMeta::GotoLine {
                line: one_based - 1,
            };
        }
    }

    /// Refresh the Move navigator's `New folder…` row from the current query
    /// — mirrors [`Self::sync_goto_line_row`]'s shape (the row never moves or
    /// gets re-created; only its label changes per keystroke). Visibility is
    /// [`Self::retain_visible_rows`]'s job, gated by the same
    /// [`Self::move_dest_new_folder_target`] this reads, so the label shown
    /// and the decision to show it can never disagree.
    fn sync_move_new_folder_row(&mut self) {
        if self.kind != OverlayKind::MoveDest {
            return;
        }
        let label = match self.move_dest_new_folder_target() {
            Some(name) => format!("New folder \"{name}\"\u{2026}"),
            None => "New folder\u{2026}".to_string(),
        };
        if let Some(row) = self
            .rows
            .iter_mut()
            .find(|r| matches!(r.meta, RowMeta::NewFolder))
        {
            row.accept = label;
        }
    }

    /// THE ONE GATE for the Move navigator's `New folder…` row — read by the
    /// label sync above, [`Self::retain_visible_rows`]'s visibility check, and
    /// `actions::overlay_nav::accept_move_dest`'s create-and-move commit, so
    /// the row's wording, its presence, and what it does on Enter cannot
    /// drift apart.
    ///
    /// `None` when there is nothing safe or meaningful to create: an empty
    /// query (nothing typed yet), a name that already belongs to a folder
    /// listed at THIS level (case-insensitive — that folder is the honest
    /// answer, reachable by descending into it instead), or a name that isn't
    /// a single path segment (`/`, `\`, `.`, `..`) — Move stays bounded to the
    /// source file's owning root, and a typed `../elsewhere` or `a/b` must
    /// never ride the create-a-folder door past that bound.
    pub fn move_dest_new_folder_target(&self) -> Option<String> {
        if self.kind != OverlayKind::MoveDest {
            return None;
        }
        let q = self.query.text().trim();
        if q.is_empty() || q == "." || q == ".." || q.contains(['/', '\\']) {
            return None;
        }
        let exists = self
            .rows
            .iter()
            .any(|r| r.is_dir && r.accept.eq_ignore_ascii_case(q));
        if exists { None } else { Some(q.to_string()) }
    }

    /// `Move here` is the Move navigator's PRIMARY VERB — the card was
    /// summoned to move the file, and doing so at the level you're standing
    /// on is always a valid answer, unlike every folder row (which the typed
    /// query can filter away entirely). So it is exempt from the fuzzy
    /// filter's own drop-on-no-match, unlike [`Self::park_terminal_rows_last`]'s
    /// family: those park LAST (a fallback once real answers run out); this
    /// one is the row a bare Enter should hit AT REST, so an EMPTY query pins
    /// it FIRST (and default-selected, since `selected` already reads 0 here).
    /// The instant something is typed, a folder match or the create-a-folder
    /// row becomes the natural target of a bare Enter instead, so a
    /// NON-EMPTY query parks `Move here` last — still reachable (never
    /// dropped), just no longer first in line.
    fn pin_move_here(&self, ranked: &mut Vec<usize>) {
        if self.kind != OverlayKind::MoveDest {
            return;
        }
        let Some(mh) = self
            .rows
            .iter()
            .position(|r| matches!(r.meta, RowMeta::MoveHere))
        else {
            return;
        };
        ranked.retain(|&ci| ci != mh);
        if self.query.text().trim().is_empty() {
            ranked.insert(0, mh);
        } else {
            ranked.push(mh);
        }
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
        // The line-jump row lives ONLY on the flat `All` lens -- it owns no
        // dedicated lens of its own (unlike Headings), and the generic
        // Files bucket (`!heading && !is_dir`) would otherwise happily claim
        // it too. Also hide it outright while the query names no valid
        // target, so a stale/placeholder label never shows.
        if self.kind == OverlayKind::Goto {
            let visible = self.facet_lens == 0 && self.goto_line_target().is_some();
            ranked.retain(|&i| !matches!(self.rows[i].meta, RowMeta::GotoLine { .. }) || visible);
        }
        // `New folder…` is the quiet create-on-unmatched-name row: present
        // only while `move_dest_new_folder_target` names a safe, genuinely
        // new folder — never while the query is empty (nothing typed to
        // create) or already names a folder listed at this level.
        if self.kind == OverlayKind::MoveDest {
            let visible = self.move_dest_new_folder_target().is_some();
            ranked.retain(|&i| !matches!(self.rows[i].meta, RowMeta::NewFolder) || visible);
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
