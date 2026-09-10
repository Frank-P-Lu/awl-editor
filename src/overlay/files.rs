//! Files card construction, presentation, and directory-level outcomes.

use std::{collections::BTreeSet, path::Path};

use super::{OverlayKind, OverlayRow, OverlayState, RowMeta};

/// The Files corpus after its one byte-aware eligibility pass.  The parallel
/// ranking vectors are rebuilt from the original positions, so filtering a
/// binary cannot point an open or recent marker at its next neighbour.
pub(crate) struct FilesCorpus {
    pub paths: Vec<String>,
    pub open: Vec<usize>,
    pub recent: Vec<usize>,
    pub times: Vec<String>,
}

/// Keep every UTF-8/NUL-free file regardless of its name, while excluding
/// binary paths from the Files card's browse and root-wide search corpus.
pub(crate) fn files_corpus(
    root: &Path,
    paths: &[String],
    open: &[usize],
    recent: &[usize],
    times: &[String],
) -> FilesCorpus {
    let mut remap = vec![None; paths.len()];
    let mut kept = Vec::new();
    let mut kept_times = Vec::new();
    for (old, path) in paths.iter().enumerate() {
        if matches!(
            crate::openable::classify(&crate::index::resolve(root, path)),
            crate::openable::Openable::Text
        ) {
            remap[old] = Some(kept.len());
            kept.push(path.clone());
            kept_times.push(times.get(old).cloned().unwrap_or_default());
        }
    }
    let remap_indices = |indices: &[usize]| {
        indices
            .iter()
            .filter_map(|&old| remap.get(old).and_then(|new| *new))
            .collect()
    };
    FilesCorpus {
        paths: kept,
        open: remap_indices(open),
        recent: remap_indices(recent),
        times: kept_times,
    }
}

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

impl OverlayState {
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
        self.facet_lens = 0;
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
