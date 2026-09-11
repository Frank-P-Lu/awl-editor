//! Focused caller-gathered inputs for [`super::build`].

/// A summoned spell-suggest picker's target: the suggestion list, the
/// misspelling's `(line, start_col, end_col)`, and the misspelled word.
pub type SpellSuggestTarget = (Vec<String>, (usize, usize, usize), String);

/// The effective key bindings shared by the Command and Keybindings pickers.
///
/// The slice borrows the caller's config because resolving the labels is part of
/// construction, not an owned picker payload.
pub struct BindingInputs<'a> {
    pub keys: &'a [(String, Vec<String>)],
    pub linux_keep: &'a [String],
    pub keymap_flavor: crate::keymap::KeymapFlavor,
}

/// The active-file data that forms Go-to's file, heading, and line-jump lenses.
pub struct GotoInputs {
    pub corpus: Vec<String>,
    pub open: Vec<usize>,
    pub recent: Vec<usize>,
    pub times: Vec<String>,
    pub headings: Vec<(String, usize)>,
    pub line_count: usize,
}

/// Command palette-only inputs: shared binding labels, its settings union, and
/// runtime row gates.
pub struct CommandInputs<'a> {
    pub bindings: BindingInputs<'a>,
    pub settings_values: crate::settings::SettingsValues,
    pub row_gates: crate::commands::RowGates,
}

/// History's rows and the caller-owned reference clocks. Ordinary replay keeps
/// both clocks absent so capture-relative lenses remain inert.
pub struct HistoryInputs {
    pub entries: Vec<crate::history::TimelineRow>,
    pub now: Option<u64>,
    pub session_start: Option<u64>,
    pub subject_name: String,
}

/// Search in folder's root and budget-bounded, summon-time corpus.
pub struct SearchFolderInputs {
    pub root: std::path::PathBuf,
    pub corpus: Vec<(String, String)>,
}

/// One focused construction request. The variant is the picker identity, so a
/// production caller cannot attach another picker's data or fill unrelated
/// fields with inert placeholders.
pub enum PickerInput<'a> {
    Goto(GotoInputs),
    Theme,
    Caret,
    Dictionary,
    CjkLang,
    Date { today_ymd: (i32, u32, u32) },
    Keymap { configured: String },
    Command(CommandInputs<'a>),
    Keybindings(BindingInputs<'a>),
    Spell(Option<SpellSuggestTarget>),
    History(HistoryInputs),
    Settings(crate::settings::SettingsValues),
    Assets(Vec<crate::assets::Orphan>),
    UserWords(Vec<String>),
    SearchFolder(SearchFolderInputs),
    Credits,
}

impl PickerInput<'_> {
    /// The only summoned picker this request can construct.
    pub fn kind(&self) -> crate::overlay::OverlayKind {
        match self {
            Self::Goto(_) => crate::overlay::OverlayKind::Goto,
            Self::Theme => crate::overlay::OverlayKind::Theme,
            Self::Caret => crate::overlay::OverlayKind::Caret,
            Self::Dictionary => crate::overlay::OverlayKind::Dictionary,
            Self::CjkLang => crate::overlay::OverlayKind::CjkLang,
            Self::Date { .. } => crate::overlay::OverlayKind::Date,
            Self::Keymap { .. } => crate::overlay::OverlayKind::Keymap,
            Self::Command(_) => crate::overlay::OverlayKind::Command,
            Self::Keybindings(_) => crate::overlay::OverlayKind::Keybindings,
            Self::Spell(_) => crate::overlay::OverlayKind::Spell,
            Self::History(_) => crate::overlay::OverlayKind::History,
            Self::Settings(_) => crate::overlay::OverlayKind::Settings,
            Self::Assets(_) => crate::overlay::OverlayKind::Assets,
            Self::UserWords(_) => crate::overlay::OverlayKind::UserWords,
            Self::SearchFolder(_) => crate::overlay::OverlayKind::SearchFolder,
            Self::Credits => crate::overlay::OverlayKind::Credits,
        }
    }
}
