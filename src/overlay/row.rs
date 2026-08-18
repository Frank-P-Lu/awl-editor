//! Picker row data and its exhaustive metadata taxonomy.

pub fn add_to_dictionary_label(word: &str) -> String {
    format!("Add '{word}' to dictionary")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayRow {
    pub accept: String,
    pub secondary: String,
    pub is_dir: bool,
    pub git: bool,
    pub meta: RowMeta,
    pub range: Option<RangeCell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeCell {
    pub id: crate::settings::SettingId,
    pub step: u16,
}

impl OverlayRow {
    pub(super) fn plain(accept: String) -> Self {
        Self {
            accept,
            secondary: String::new(),
            is_dir: false,
            git: false,
            meta: RowMeta::Plain,
            range: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowMeta {
    Plain,
    GotoFile {
        time: String,
    },
    GotoHeading {
        line: usize,
    },
    /// Go to Line's own destination row -- the Headings lens's numeric
    /// companion. Synthesized/refreshed live from the typed query
    /// (`OverlayState::attach_line_jump` appends the one slot,
    /// `OverlayState::refilter`'s `sync_goto_line_row` step keeps its label
    /// and `line` in step with the query on every keystroke); `line` is
    /// already clamped to the destination buffer's own line count and
    /// zero-based, matching `GotoHeading`.
    GotoLine {
        line: usize,
    },
    /// A folder destination in the unified Go-to roster. Its absolute path is
    /// carried in `OverlayRow::accept`; accepting it switches the active writing
    /// folder through the same typed `OverlayAccept(Project, ..)` effect as the
    /// older folder picker.
    GotoFolder,
    FolderChooser,
    CommandSetting {
        id: crate::settings::SettingId,
    },
    CommandHidden,
    SpellAdd,
    History {
        id: String,
        ts: u64,
    },
    /// The flat switch-project picker's door row. Metadata, rather than its
    /// wording, identifies the row that opens `OverlayKind::ProjectBrowse`.
    ProjectDoor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // consumed by OverlayKind::row_meta_roster + overlay laws.
pub enum RowMetaTag {
    Plain,
    GotoFile,
    GotoHeading,
    GotoLine,
    GotoFolder,
    FolderChooser,
    CommandSetting,
    CommandHidden,
    SpellAdd,
    History,
    ProjectDoor,
}

impl RowMeta {
    #[allow(dead_code)] // exhaustive metadata witness used by overlay laws.
    pub fn tag(&self) -> RowMetaTag {
        match self {
            RowMeta::Plain => RowMetaTag::Plain,
            RowMeta::GotoFile { .. } => RowMetaTag::GotoFile,
            RowMeta::GotoHeading { .. } => RowMetaTag::GotoHeading,
            RowMeta::GotoLine { .. } => RowMetaTag::GotoLine,
            RowMeta::GotoFolder => RowMetaTag::GotoFolder,
            RowMeta::FolderChooser => RowMetaTag::FolderChooser,
            RowMeta::CommandSetting { .. } => RowMetaTag::CommandSetting,
            RowMeta::CommandHidden => RowMetaTag::CommandHidden,
            RowMeta::SpellAdd => RowMetaTag::SpellAdd,
            RowMeta::History { .. } => RowMetaTag::History,
            RowMeta::ProjectDoor => RowMetaTag::ProjectDoor,
        }
    }

    /// A row that must stay last because it acts on something other than the
    /// query. `OverlayState::refilter` is the one consumer of this taxonomy.
    pub fn terminal(&self) -> bool {
        match self {
            RowMeta::SpellAdd
            | RowMeta::ProjectDoor
            | RowMeta::FolderChooser
            | RowMeta::GotoLine { .. } => true,
            RowMeta::Plain
            | RowMeta::GotoFile { .. }
            | RowMeta::GotoHeading { .. }
            | RowMeta::GotoFolder
            | RowMeta::CommandSetting { .. }
            | RowMeta::CommandHidden
            | RowMeta::History { .. } => false,
        }
    }
}
