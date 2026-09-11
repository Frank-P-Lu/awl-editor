//! Files card construction, presentation, and directory-level outcomes.

use std::{collections::BTreeSet, path::Path};

use super::{OverlayKind, OverlayRow, OverlayState, RowMeta};

/// The keyboard target inside the Files card. Selection remains a separate fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesFocus {
    Query,
    Files,
    Recent,
    Up,
    Choices,
    ChangeFolder,
    NewDocument,
}

/// Byte-classify only the displayed directory level, returning the same
/// root-relative identities the Files corpus uses. Live App and replay call
/// this owner so the bounded-I/O and path-spelling rules cannot drift.
pub(crate) fn unsupported_level_files(
    root: &Path,
    rel: Option<&str>,
    entries: &[crate::index::DirEntry],
) -> BTreeSet<String> {
    let prefix = rel
        .filter(|path| !path.is_empty())
        .map(|path| format!("{path}/"));
    entries
        .iter()
        .filter(|entry| !entry.is_dir)
        .filter_map(|entry| {
            let relative = match &prefix {
                Some(prefix) => format!("{prefix}{}", entry.name),
                None => entry.name.clone(),
            };
            matches!(
                crate::openable::classify(&crate::index::resolve(root, &relative)),
                crate::openable::Openable::Unsupported { .. }
            )
            .then_some(relative)
        })
        .collect()
}

impl OverlayState {
    /// Remove binary leaves discovered at the currently displayed directory
    /// level.  The root-wide index stays unread until a level reaches it;
    /// ranking indices follow the surviving rows rather than drifting.
    pub fn exclude_files(&mut self, paths: &BTreeSet<String>) {
        if !self.files_mode || paths.is_empty() {
            return;
        }
        let mut remap = vec![None; self.rows.len()];
        let mut rows = Vec::with_capacity(self.rows.len());
        for (old, row) in self.rows.drain(..).enumerate() {
            if matches!(row.meta, RowMeta::GotoFile { .. }) && paths.contains(&row.accept) {
                continue;
            }
            remap[old] = Some(rows.len());
            rows.push(row);
        }
        let remap_indices = |indices: &[usize]| {
            indices
                .iter()
                .filter_map(|&old| remap.get(old).and_then(|new| *new))
                .collect()
        };
        self.open = remap_indices(&self.open);
        self.recent = remap_indices(&self.recent);
        self.rows = rows;
        self.refilter();
        self.refresh_hug_roster();
    }

    pub fn new_files(
        files: Vec<String>,
        open: Vec<usize>,
        recent: Vec<usize>,
        browse_dir: Option<String>,
    ) -> Self {
        let mut rows = files
            .iter()
            .map(|file| OverlayRow {
                accept: file.clone(),
                secondary: String::new(),
                is_dir: false,
                git: false,
                meta: RowMeta::GotoFile {
                    time: String::new(),
                },
                range: None,
            })
            .collect::<Vec<_>>();
        rows.extend(file_ancestors(&files).into_iter().map(folder_row));
        let mut change = OverlayRow::plain("Change folder…".to_string());
        change.meta = RowMeta::FolderChooser;
        rows.push(change);
        let destination = browse_dir
            .as_deref()
            .filter(|dir| !dir.is_empty())
            .unwrap_or("root");
        let mut new_doc = OverlayRow::plain(format!("New document — {destination}/"));
        new_doc.meta = RowMeta::NewDocument;
        rows.push(new_doc);

        let mut state = Self::new_marked(
            OverlayKind::Goto,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            browse_dir,
        );
        state.rows = rows;
        state.files_mode = true;
        state.open = open;
        state.recent = recent;
        state.refilter();
        state.refresh_hug_roster();
        state
    }

    /// Add level-reader identities so genuinely empty directories remain visible.
    pub fn attach_file_directories(&mut self, directories: Vec<String>) {
        if !self.files_mode {
            return;
        }
        for path in directories {
            if !self.rows.iter().any(|row| row.is_dir && row.accept == path) {
                self.rows.push(folder_row(path));
            }
        }
        self.refilter();
        self.refresh_hug_roster();
    }

    /// Preserve unreadable, truly empty, and unsupported-only outcomes separately.
    pub fn set_files_level_state(&mut self, level: Option<&[crate::index::DirEntry]>) {
        if !self.files_mode {
            return;
        }
        self.notice = match level {
            None => "folder unavailable — check access and try again".into(),
            Some([]) => "this folder is empty".into(),
            Some(entries) if unsupported_only(self, entries) => {
                "no supported files in this folder".into()
            }
            Some(_) => String::new(),
        };
    }

    pub fn files_destination(&self) -> Option<String> {
        (self.kind == OverlayKind::Goto).then(|| self.browse_dir.clone().unwrap_or_default())
    }

    pub(super) fn files_title(&self) -> Option<String> {
        (self.kind == OverlayKind::Goto).then(|| {
            self.browse_dir
                .as_deref()
                .filter(|dir| !dir.is_empty())
                .map(|dir| format!("files  /  {}/", dir.replace('/', "  /  ")))
                .unwrap_or_else(|| "files  /".to_string())
        })
    }

    pub(super) fn files_hint(&self) -> Option<String> {
        if self.kind != OverlayKind::Goto || self.goto_outline_only {
            return None;
        }
        use FilesFocus::*;
        match self.files_focus {
            Files => return Some("Files view   ↵ show   tab next   esc close".into()),
            Recent if self.recent.is_empty() => {
                return Some("no recent files yet   tab next   esc close".into());
            }
            Recent => return Some("Recent view   ↵ show   tab next   esc close".into()),
            Up => return Some("Up   ↵ ascend   tab next   esc close".into()),
            Query | Choices | ChangeFolder | NewDocument => {}
        }
        let verb = self
            .selected_corpus_index()
            .and_then(|index| self.rows.get(index))
            .map(|row| match row.meta {
                RowMeta::GotoFolder => "enter folder",
                RowMeta::GotoFile { .. } => "open file",
                RowMeta::FolderChooser => "change folder",
                RowMeta::NewDocument => "new document",
                _ => "choose",
            })
            .unwrap_or("choose");
        let up = if self.browse_dir.is_some() {
            "   ← up"
        } else {
            ""
        };
        Some(format!("type to search   ↵ {verb}{up}   esc close"))
    }

    pub fn focus_headings(&mut self) {
        self.goto_outline_only = true;
        self.focus_facet_id("headings");
        self.query = crate::textbox::TextBox::new();
        self.selected = 0;
        self.scroll = 0;
        self.refilter();
    }

    pub fn set_times(&mut self, times: Vec<String>) {
        let mut file_index = 0;
        for row in &mut self.rows {
            if let RowMeta::GotoFile { time } = &mut row.meta {
                *time = times.get(file_index).cloned().unwrap_or_default();
                file_index += 1;
            }
        }
        self.refresh_hug_roster();
    }

    /// Compatibility corpus for the retired unified Goto folder lens.
    #[allow(dead_code)]
    pub fn attach_folders(&mut self, folders: Vec<(String, bool)>, recent_paths: &[String]) {
        if self.kind != OverlayKind::Goto {
            return;
        }
        let start = self.rows.len();
        for (path, is_git) in folders {
            let mut row = folder_row(path);
            row.git = is_git;
            self.rows.push(row);
        }
        for path in recent_paths {
            if let Some(index) = self.rows[start..]
                .iter()
                .position(|row| &row.accept == path)
                .map(|index| start + index)
                && !self.recent.contains(&index)
            {
                self.recent.push(index);
            }
        }
        let mut chooser = OverlayRow::plain("Choose another folder…".to_string());
        chooser.meta = RowMeta::FolderChooser;
        self.rows.push(chooser);
        self.refilter();
        self.refresh_hug_roster();
    }
}

fn file_ancestors(files: &[String]) -> BTreeSet<String> {
    let mut dirs = BTreeSet::new();
    for file in files {
        let mut path = std::path::Path::new(file).parent();
        while let Some(parent) = path {
            let rel = parent.to_string_lossy().replace('\\', "/");
            if rel.is_empty() || rel == "." {
                break;
            }
            dirs.insert(rel);
            path = parent.parent();
        }
    }
    dirs
}

fn folder_row(path: String) -> OverlayRow {
    let mut row = OverlayRow::plain(path);
    row.is_dir = true;
    row.meta = RowMeta::GotoFolder;
    row.secondary = "folder".into();
    row
}

fn unsupported_only(state: &OverlayState, entries: &[crate::index::DirEntry]) -> bool {
    entries.iter().any(|entry| !entry.is_dir)
        && !state
            .items
            .iter()
            .any(|&index| matches!(state.rows[index].meta, RowMeta::GotoFile { .. }))
}
