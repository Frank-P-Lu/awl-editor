//! [`BuildCtx`] — the caller-gathered inputs [`super::build`] needs.

/// A summoned spell-suggest picker's target: the suggestion list, the
/// misspelling's `(line, start_col, end_col)`, and the misspelled word.
/// Shared with the replay chord layer, which gathers this same value for
/// [`BuildCtx::spell_target`] before a summon.
pub type SpellSuggestTarget = (Vec<String>, (usize, usize, usize), String);

/// Caller-gathered inputs for [`super::build`], shared by the live App and replay.
/// Ordinary replay omits live recency data to keep captures deterministic.
pub struct BuildCtx<'a> {
    /// Root-relative Go-to paths; live default-folder listings are recency-ordered.
    pub goto_corpus: Vec<String>,
    /// Open `goto_corpus` indices for ranking; empty in ordinary replay.
    pub goto_open: Vec<usize>,
    /// Recently opened `goto_corpus` indices for ranking; empty in ordinary replay.
    pub goto_recent: Vec<usize>,
    /// Time labels parallel to `goto_corpus`; blank outside the live default folder,
    /// empty in ordinary replay.
    pub goto_times: Vec<String>,
    /// Config `[keys]` overrides for effective binding labels.
    pub config_keys: &'a [(String, Vec<String>)],
    /// Effective Linux keep chords, including built-in defaults; inert on Mac.
    pub config_linux_keep: &'a [String],
    /// Configured keymap flavor for effective binding resolution.
    pub config_keymap_flavor: crate::keymap::KeymapFlavor,
    /// Current Markdown headings as `(indented label, zero-based line)` for Go-to.
    pub goto_headings: Vec<(String, usize)>,
    /// Current buffer line count for Go to Line; zero disables the row.
    pub goto_line_count: usize,
    /// Absolute Go-to folder destinations and their Git markers.
    pub goto_folders: Vec<(String, bool)>,
    /// Newest-first folder MRU for Go-to's Recent lens.
    pub goto_recent_folders: Vec<String>,
    /// Spell-picker input; `None` leaves a spell summon unopened.
    pub spell_target: Option<SpellSuggestTarget>,
    /// Current file history rows, newest-first; gathered for History or Compare.
    pub history_entries: Vec<crate::history::TimelineRow>,
    /// Reference time in milliseconds for History's clock-relative lenses;
    /// `None` in ordinary replay keeps those lenses inert.
    pub history_now: Option<u64>,
    /// Session start in milliseconds; `None` before session tracking starts
    /// or in ordinary replay.
    pub history_session_start: Option<u64>,
    /// Config and project inputs for Settings cells; the readout reads
    /// process-global settings directly.
    pub settings_values: crate::settings::SettingsValues,
    /// Orphans scanned when opening Asset Cleaner; empty for other actions.
    pub assets: Vec<crate::assets::Orphan>,
    /// Alphabetical personal-dictionary words. Ordinary replay supplies an empty
    /// list; `--screenshot-app` reaches the App's dictionary loading path.
    pub user_words: Vec<String>,
    /// Runtime facts controlling conditional command rows; defaulted in ordinary replay.
    pub row_gates: crate::commands::RowGates,
    /// Root and budget-bounded corpus for Search in folder, loaded once at summon.
    pub search_root: std::path::PathBuf,
    pub search_corpus: Vec<(String, String)>,
}
