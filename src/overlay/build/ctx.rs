//! [`BuildCtx`] — the caller-gathered inputs [`super::build`] needs.

/// A summoned spell-suggest picker's target: the suggestion list, the
/// misspelling's `(line, start_col, end_col)`, and the misspelled word.
/// Shared with the replay chord layer, which gathers this same value for
/// [`BuildCtx::spell_target`] before a summon.
pub type SpellSuggestTarget = (Vec<String>, (usize, usize, usize), String);

/// The inputs the FLAT-picker overlay builder ([`super::build`]) needs, gathered by the
/// caller so the construction itself lives in ONE place (shared by the live App
/// and the headless `--keys` replay). The live-only GO-TO recency bits
/// (`goto_open` / `goto_recent` / `goto_times`) are filled by the App and left
/// EMPTY by the headless path, keeping the capture byte-stable. `config_keys`
/// feeds the command palette's EFFECTIVE bindings.
pub struct BuildCtx<'a> {
    /// The go-to corpus (root-relative paths), already recency-ordered when live.
    pub goto_corpus: Vec<String>,
    /// Corpus indices currently OPEN — ranking bias (live-only; empty headless).
    pub goto_open: Vec<usize>,
    /// Corpus indices recently opened — ranking bias (live-only; empty headless).
    pub goto_recent: Vec<usize>,
    /// Per-file "last edited" labels, parallel to `goto_corpus` (live-only; empty
    /// for a non-notes root AND in headless capture, for determinism).
    pub goto_times: Vec<String>,
    /// Config `[keys]` overrides → the command palette's effective binding column.
    pub config_keys: &'a [(String, Vec<String>)],
    /// Config `linux_keep_emacs` — the per-chord door that keeps a kept chord's
    /// emacs meaning showing (and suppresses the native label it would otherwise
    /// display) in the SAME effective binding column, under `Convention::Linux`
    /// only (see `commands::join_slots_truthful`'s Tier 4). Empty on Mac and on
    /// every headless capture that doesn't pass `--config`.
    pub config_linux_keep: &'a [String],
    /// The config `keymap` flavor, beside [`Self::config_linux_keep`] — the
    /// command palette's chord column falls back to a seeded layer chord
    /// (`commands::menu_native_label`'s doc) when no ordinary chord survives
    /// under `keymap = "emacs"`, and needs to know which flavor is active to
    /// ask `commands::seeded_chords_for` the right question.
    pub config_keymap_flavor: crate::keymap::KeymapFlavor,
    /// The CURRENT buffer's markdown headings (depth-indented label + line) for
    /// Go-to's HEADINGS lens (the fold that retired the standalone Outline picker).
    /// Caller-gathered (it needs the live buffer text); EMPTY for a non-markdown
    /// buffer or one with no headings, so the Headings lens simply reads empty.
    pub goto_headings: Vec<(String, usize)>,
    /// The CURRENT buffer's total line count for Go-to's LINE-JUMP row (Go to
    /// Line, the Headings lens's numeric companion). Unlike `goto_headings`
    /// this is gathered for ANY buffer, not just a markdown one -- prose and
    /// light code both address by line number. `0` for a non-Go-to summon and
    /// for a headless fixture that doesn't gather it, the harmless "no
    /// line-jump offered" default `OverlayState::attach_line_jump` treats as
    /// opt-out.
    pub goto_line_count: usize,
    /// Absolute folder destinations for Go-to, each with its git marker.
    pub goto_folders: Vec<(String, bool)>,
    /// Absolute folder MRU, newest-first. `attach_folders` enrols these into the
    /// shared Recent lens after the file MRU.
    pub goto_recent_folders: Vec<String>,
    /// The Cmd-`;` spell target — the misspelled word's corrections, its span, AND
    /// its current TEXT — resolved by the caller ONLY when the spell binding fired.
    /// `None` when the cursor isn't on a flagged word (or spell-check is off), so
    /// the summon no-ops. The word text builds the "Add '<word>' to dictionary" row
    /// label + rides the add-row accept effect ([`crate::overlay::OverlayState::new_spell`]).
    pub spell_target: Option<SpellSuggestTarget>,
    /// The HISTORY TIMELINE rows for the current file — [`crate::history::TimelineRow`]
    /// (when / which / counts / id), newest-first — resolved by the caller (via
    /// [`crate::history::timeline_rows`]) ONLY when the History binding fired. EMPTY
    /// otherwise AND when the file has no history yet; an empty list summons the calm
    /// "no history yet" row (History always opens; the Headings lens simply reads empty).
    pub history_entries: Vec<crate::history::TimelineRow>,
    /// The REFERENCE clock (millis) for the History picker's Today lens — `Some`
    /// live, `None` in the headless capture path (so the clock-relative lenses stay
    /// inert, the determinism gate).
    pub history_now: Option<u64>,
    /// The current session's start (millis) for the History picker's Session lens —
    /// `Some` live, `None` headless / untracked.
    pub history_session_start: Option<u64>,
    /// The config/project-derived VALUE inputs for the SETTINGS menu's secondary
    /// column ([`crate::settings::SettingsValues`]). The process-global settings
    /// (theme / page mode / caret / spell / markdown / nits) are read LIVE inside
    /// the readout, so only the config pieces are gathered by the caller — the live
    /// App from `self.config` + root + zoom, the headless replay from its `config`.
    /// Empty [`Default`] for a non-Settings summon (unused there).
    pub settings_values: crate::settings::SettingsValues,
    /// The ASSET CLEANER's scanned ORPHAN list ([`crate::assets::scan`]) — filled by
    /// the caller ONLY when the "Clean unused assets" binding fired (scanning the whole
    /// project tree is pure waste otherwise), EMPTY for every other summon. The live
    /// App AND the headless replay both fill it from the same scan over the
    /// [`crate::fs`] seam, so a `--keys` capture sees the real orphan list.
    pub assets: Vec<crate::assets::Orphan>,
    /// The PERSONAL DICTIONARY's words, alphabetical
    /// ([`crate::spell::SpellChecker::user_words_sorted`]) — filled by the
    /// caller ONLY when the "Personal dictionary" binding fired, EMPTY for
    /// every other summon, exactly like `assets` above. Both the live App and
    /// the headless replay fill it from the same live checker, so a `--keys`
    /// capture sees the real word list.
    pub user_words: Vec<String>,
    /// Is a daemon `--wait` client actively waiting on the CURRENT buffer right
    /// now (`crate::daemon`'s module doc, `App::wait_conns`)? The ONE live fact
    /// behind the Command palette's "Finish file" row visibility
    /// (`commands::visible_hidden_mask`) — `true` only on the live App, when the
    /// daemon exists AND some connection is parked waiting. Structurally `false`
    /// in the headless capture/replay path (which never imports `crate::daemon`
    /// at all — the daemon capture gate) and on wasm/`mas` (no daemon compiled),
    /// so a default palette build hides the row deterministically everywhere but
    /// a real `EDITOR=awl --wait` round-trip.
    pub row_gates: crate::commands::RowGates,
    /// "Search in folder…"'s root + its already-loaded, budget-bounded corpus
    /// (`crate::search_folder::load_corpus` over `crate::index::build_index`'s
    /// own gitignore-aware roster) — filled by the caller ONLY when the search
    /// binding fired (reading every file's content is pure waste otherwise),
    /// EMPTY for every other summon. The live App AND the headless replay both
    /// fill it from the same load over the [`crate::fs`] seam, so a `--keys`
    /// capture sees the real corpus. The picker itself re-matches this same
    /// in-memory corpus against the typed query on every keystroke
    /// (`OverlayState::refilter`'s `SearchFolder` branch); this is gathered
    /// once, at summon, never on a keystroke.
    pub search_root: std::path::PathBuf,
    pub search_corpus: Vec<(String, String)>,
}
