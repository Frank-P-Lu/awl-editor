//! Settings corpus, categories, and value readouts for the shared overlay.

use crate::facets::{Facet, FacetItem, FacetScheme};
use std::path::Path;
mod scroll_sensitivity;

pub fn scroll_sensitivity() -> f32 {
    scroll_sensitivity::get()
}

pub fn set_scroll_sensitivity(value: f32) {
    scroll_sensitivity::set(value);
}

/// How a setting is EDITED (drives what Enter does). Carried as DATA on each
/// [`SettingRow`], never a code path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKind {
    Toggle,
    Picker,
    #[allow(dead_code)] // retained grammar; all current bounded numerics use Range
    Value,
    Range,
    Path,
    Submenu,
    Action,
}

/// The CLOSED identity of a settings row — the ONE key every behavior lookup
/// (value readout, config-key maps, sub-overlay map, Action dispatch, the
/// Command-palette settings-row resolution) switches on. 1:1 with [`SETTINGS`]
/// in table order (enforced by [`tests::every_setting_id_maps_1_to_1_to_the_registry`]).
/// [`SettingRow::name`] is the DISPLAY LABEL only — renaming a row's label can
/// never re-route or drop its behavior, because no behavior map keys on it
/// anymore (the bug class [`tests::a_label_edit_changes_no_behavior`] guards).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingId {
    CaretStyle,
    PageMode,
    TypewriterScroll,
    ReduceMotion,
    PageWidthProse,
    PageWidthCode,
    Zoom,
    ScrollSensitivity,
    DateFormat,
    Theme,
    Wysiwyg,
    FormatPopover,
    InlineImages,
    CodeLigatures,
    Outline,
    MenuBar,
    Spellcheck,
    Dictionary,
    WritingNits,
    CjkReadsAs,
    DefaultFolder,
    ProjectsFolder,
    ProjectRoot,
    FileVisibility,
    Autosave,
    LocalHistory,
    SessionRestore,
    Keymap,
    Keybindings,
    ReportProblem,
    EditConfigAsText,
}

/// One row of the settings corpus: its TYPED [`id`](SettingId) (the one behavior
/// key), its display `name` (PRESENTATION ONLY — the fuzzy corpus text and the
/// row's drawn label, never a lookup key), the `category` it buckets under
/// (also a lens SECTION label — see [`SETTINGS_FACET_STRIP`]), and its
/// [`SettingKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingRow {
    pub id: SettingId,
    pub name: &'static str,
    pub category: &'static str,
    pub kind: SettingKind,
}

/// The 31-setting corpus, in stable display order (grouped by category). The ONE
/// owner — the FacetScheme bucket + the value readout both key off this table.
pub static SETTINGS: &[SettingRow] = &[
    SettingRow {
        id: SettingId::CaretStyle,
        name: "Caret style",
        category: "Editor",
        kind: SettingKind::Picker,
    },
    SettingRow {
        id: SettingId::PageMode,
        name: "Page mode",
        category: "Editor",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::TypewriterScroll,
        name: "Typewriter scroll",
        category: "Editor",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::ReduceMotion,
        name: "Reduce motion",
        category: "Editor",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::PageWidthProse,
        name: "Page width (prose)",
        category: "Editor",
        kind: SettingKind::Range,
    },
    SettingRow {
        id: SettingId::PageWidthCode,
        name: "Page width (code)",
        category: "Editor",
        kind: SettingKind::Range,
    },
    SettingRow {
        id: SettingId::Zoom,
        name: "Zoom",
        category: "Editor",
        kind: SettingKind::Range,
    },
    SettingRow {
        id: SettingId::ScrollSensitivity,
        name: "Scroll sensitivity",
        category: "Editor",
        kind: SettingKind::Range,
    },
    SettingRow {
        id: SettingId::DateFormat,
        name: "Date format",
        category: "Editor",
        kind: SettingKind::Picker,
    },
    SettingRow {
        id: SettingId::Theme,
        name: "Theme",
        category: "Appearance",
        kind: SettingKind::Picker,
    },
    SettingRow {
        id: SettingId::Wysiwyg,
        name: "WYSIWYG",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::FormatPopover,
        name: "Format popover",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::InlineImages,
        name: "Inline images",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::CodeLigatures,
        name: "Code ligatures",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Outline,
        name: "Outline",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::MenuBar,
        name: "Menu bar",
        category: "Appearance",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Spellcheck,
        name: "Spellcheck",
        category: "Writing",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Dictionary,
        name: "Dictionary",
        category: "Writing",
        kind: SettingKind::Picker,
    },
    SettingRow {
        id: SettingId::WritingNits,
        name: "Writing nits",
        category: "Writing",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::CjkReadsAs,
        name: "Ambiguous CJK reads as",
        category: "Writing",
        kind: SettingKind::Picker,
    },
    SettingRow {
        id: SettingId::DefaultFolder,
        name: "Default folder",
        category: "Files",
        kind: SettingKind::Path,
    },
    SettingRow {
        id: SettingId::ProjectsFolder,
        name: "Projects folder",
        category: "Files",
        kind: SettingKind::Path,
    },
    SettingRow {
        id: SettingId::ProjectRoot,
        name: "Project root",
        category: "Files",
        kind: SettingKind::Path,
    },
    SettingRow {
        id: SettingId::FileVisibility,
        name: "File visibility",
        category: "Files",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Autosave,
        name: "Autosave",
        category: "Files",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::LocalHistory,
        name: "Local history",
        category: "Files",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::SessionRestore,
        name: "Session restore",
        category: "Files",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Keymap,
        name: "Keymap",
        category: "Keybindings",
        kind: SettingKind::Toggle,
    },
    SettingRow {
        id: SettingId::Keybindings,
        name: "Keybindings",
        category: "Keybindings",
        kind: SettingKind::Submenu,
    },
    SettingRow {
        id: SettingId::ReportProblem,
        name: "Report a Problem",
        category: "Advanced",
        kind: SettingKind::Action,
    },
    SettingRow {
        id: SettingId::EditConfigAsText,
        name: "Edit config as text",
        category: "Advanced",
        kind: SettingKind::Action,
    },
];

pub fn row_of(id: SettingId) -> SettingRow {
    *SETTINGS
        .iter()
        .find(|r| r.id == id)
        .expect("every SettingId has a row — see every_setting_id_maps_1_to_1_to_the_registry")
}

static SETTINGS_FACET_STRIP: [Facet; 7] = [
    Facet {
        label: "All",
        id: "all",
        sections: &[],
    },
    Facet {
        label: "Editor",
        id: "editor",
        sections: &["Editor"],
    },
    Facet {
        label: "Appearance",
        id: "appearance",
        sections: &["Appearance"],
    },
    Facet {
        label: "Writing",
        id: "writing",
        sections: &["Writing"],
    },
    Facet {
        label: "Files",
        id: "files",
        sections: &["Files"],
    },
    Facet {
        label: "Keybindings",
        id: "keybindings",
        sections: &["Keybindings"],
    },
    Facet {
        label: "Advanced",
        id: "advanced",
        sections: &["Advanced"],
    },
];

/// The category a setting name buckets under, or `None` for an unknown name. Looks
/// the row up in the single-owner [`SETTINGS`] table.
pub fn category_of(name: &str) -> Option<&'static str> {
    SETTINGS.iter().find(|r| r.name == name).map(|r| r.category)
}

/// The settings menu's [`FacetScheme::bucket`], keyed by strip index. Each
/// refinement lens (≥ 1) names exactly one category section; a row is placed under
/// it iff its own category ([`category_of`]) matches that section. Never called for
/// strip index 0 (the flat All home).
fn settings_bucket(item: FacetItem, lens_idx: usize) -> Option<&'static str> {
    let section = SETTINGS_FACET_STRIP.get(lens_idx)?.sections.first()?;
    (category_of(item.accept) == Some(*section)).then_some(*section)
}

pub static SETTINGS_FACETS: FacetScheme = FacetScheme {
    strip: &SETTINGS_FACET_STRIP,
    bucket: settings_bucket,
};

/// The CONFIG/PROJECT-derived value inputs for the settings readout — the pieces
/// that are NOT a process-global (so [`value_for`] can't read them straight). The
/// process-global settings (theme / page mode / caret / spell / markdown / nits)
/// are read live inside [`value_for`]; these come from the caller's `Config` +
/// active project root + zoom, gathered once at overlay-build time so the live App
/// and the headless replay produce identical value cells. Empty [`Default`] for the
/// non-Settings build sites (which never construct a Settings overlay).
#[derive(Clone, Debug, Default)]
pub struct SettingsValues {
    pub page_width_prose: usize,
    pub page_width_code: usize,
    pub zoom: f32,
    pub scroll_sensitivity: f32,
    pub default_folder: String,
    pub workspace: String,
    pub project_root: String,
    pub autosave: bool,
    pub history: bool,
    pub session_restore: bool,
    pub keymap: String,
    /// TODAY as a UTC civil `(year, month, day)`, for the "Date format" row's
    /// live preview ("what you see is what inserts") — gathered like
    /// `history_now`/`history_session_start` (`overlay::BuildCtx`), because
    /// [`value_for`] can't tell live from headless capture itself: the live
    /// caller passes [`crate::dateformat::today_from_system_clock`]'s real
    /// result, the headless capture/replay path the FIXED
    /// [`crate::dateformat::CAPTURE_PLACEHOLDER_YMD`] — the determinism gate
    /// that keeps a `--keys "Cmd-,"` Settings capture byte-stable.
    pub today_ymd: (i32, u32, u32),
}

impl SettingsValues {
    /// Gather the config/project-derived value inputs from the caller's `config`,
    /// the active `project_root`, the current `zoom`, and the caller's OWN
    /// `today_ymd` (real live clock, or the fixed headless placeholder — see the
    /// field doc). Everything else is read live from the process-globals inside
    /// [`value_for`] — INCLUDING the "Ambiguous CJK reads as" row now
    /// (`crate::frontmatter::cjk_priority()`, like Theme/Dictionary) and "Date
    /// format" itself (`crate::dateformat::active_format()`), so neither carries
    /// a field here.
    pub fn gather(
        config: &crate::config::Config,
        project_root: &Path,
        zoom: f32,
        today_ymd: (i32, u32, u32),
    ) -> Self {
        let path_or_dash = |p: &Option<std::path::PathBuf>| {
            p.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "—".to_string())
        };
        Self {
            page_width_prose: config.measure_for(crate::page::PageClass::Prose),
            page_width_code: config.measure_for(crate::page::PageClass::Code),
            zoom,
            scroll_sensitivity: config
                .scroll_sensitivity
                .unwrap_or(crate::range::SCROLL_SENSITIVITY.default),
            default_folder: path_or_dash(&config.default_folder),
            workspace: path_or_dash(&config.workspace),
            project_root: project_root.display().to_string(),
            autosave: config.autosave_on(),
            history: config.history_on(),
            session_restore: config.session_restore_on(),
            keymap: config.keymap_flavor().config_name().to_string(),
            today_ymd,
        }
    }
}

fn on_off(b: bool) -> &'static str {
    if b { "on" } else { "off" }
}

/// The single no-wildcard SettingId → live readout mapping.
pub fn value_for(row: &SettingRow, values: &SettingsValues) -> String {
    match row.id {
        SettingId::CaretStyle => crate::caret::mode().label().to_string(),
        SettingId::PageMode => on_off(crate::page::page_on()).to_string(),
        SettingId::TypewriterScroll => on_off(crate::typewriter::typewriter_on()).to_string(),
        SettingId::ReduceMotion => on_off(crate::motion::reduced()).to_string(),
        SettingId::PageWidthProse => {
            crate::range::PAGE_WIDTH_PROSE.format(values.page_width_prose as f32)
        }
        SettingId::PageWidthCode => {
            crate::range::PAGE_WIDTH_CODE.format(values.page_width_code as f32)
        }
        // ZOOM (item 94): formatted by its own RANGE SPEC's display unit — the
        // SAME owner the rail, the sidecar and the exact-entry parse read, so the
        // cell and the thumb can never disagree about the value.
        SettingId::Zoom => crate::range::ZOOM.format(values.zoom),
        SettingId::ScrollSensitivity => {
            crate::range::SCROLL_SENSITIVITY.format(values.scroll_sensitivity)
        }
        // DATE FORMAT: the active process-global format, rendered against the
        // caller-gathered TODAY (real live clock / the fixed headless
        // placeholder — see `SettingsValues::today_ymd`'s doc) — "what you see
        // is what inserts".
        SettingId::DateFormat => {
            let (y, m, d) = values.today_ymd;
            crate::dateformat::active_format().format(y, m, d)
        }
        SettingId::Theme => crate::theme::active().name.to_string(),
        SettingId::Wysiwyg => on_off(crate::markdown::wysiwyg_on()).to_string(),
        SettingId::FormatPopover => on_off(crate::popover::popover_on()).to_string(),
        SettingId::InlineImages => on_off(crate::markdown::inline_images_on()).to_string(),
        SettingId::CodeLigatures => on_off(crate::render::code_ligatures_on()).to_string(),
        // Outline + Menu bar read their PROCESS GLOBALS live — the SAME owners the
        // renderer reads (`outline_layout` / the bar strip) and the SAME owners
        // `App::setting_toggle` flips, like "Page mode"/"WYSIWYG"/"Spellcheck" above.
        // (They used to read config-gathered copies, which the toggle's `persist_pref`
        // mirror kept in step ONLY when a config path exists — on web, with no config
        // file, the toggle flipped the renderer but not the readout. Caught by the
        // every-toggle-dispatches sweep; both owners now agree by construction. The
        // capture path agrees too: `apply_sticky_globals` seeds these globals from
        // `--config` at every launch, live and headless alike.)
        SettingId::Outline => on_off(crate::outline::outline_on()).to_string(),
        SettingId::MenuBar => on_off(crate::menubar::menu_bar_on()).to_string(),
        SettingId::Spellcheck => on_off(crate::spell::spellcheck_on()).to_string(),
        SettingId::Dictionary => crate::spell::active_variant().label().to_string(),
        SettingId::WritingNits => on_off(crate::nits::nits_on()).to_string(),
        // The FRONT of the live ambiguity ladder, in writer-words ("Japanese",
        // never the raw BCP 47 code) — read live like Theme/Dictionary, not
        // from `values` (see `SettingsValues::gather`'s doc).
        SettingId::CjkReadsAs => crate::frontmatter::cjk_priority()
            .first()
            .map(|l| l.label().to_string())
            .unwrap_or_else(|| "—".to_string()),
        SettingId::DefaultFolder => values.default_folder.clone(),
        SettingId::ProjectsFolder => values.workspace.clone(),
        SettingId::ProjectRoot => values.project_root.clone(),
        SettingId::FileVisibility => crate::file_visibility::label().to_string(),
        SettingId::Autosave => on_off(values.autosave).to_string(),
        SettingId::LocalHistory => on_off(values.history).to_string(),
        SettingId::SessionRestore => on_off(values.session_restore).to_string(),
        SettingId::Keymap => values.keymap.clone(),
        SettingId::Keybindings | SettingId::ReportProblem | SettingId::EditConfigAsText => {
            String::new()
        }
    }
}

/// The config KEY a TOGGLE row flips + persists under — the single owner of the
/// [`SettingId`] → config-key map for the Enter-to-toggle interaction. `None` for a
/// non-toggle id (it never signals a `SettingToggle`). The RETURNED wire string is
/// UNCHANGED from before item 55 — only the ARGUMENT went from `&str` label to
/// `SettingId` — so `Config::write_pref`/`App::setting_toggle`/an old `config.toml`
/// all still see the exact same key.
pub fn toggle_key(id: SettingId) -> Option<&'static str> {
    Some(match id {
        SettingId::PageMode => "page_mode",
        SettingId::TypewriterScroll => "typewriter_scroll",
        SettingId::ReduceMotion => "reduce_motion",
        SettingId::Wysiwyg => "wysiwyg",
        SettingId::FormatPopover => "popover",
        SettingId::InlineImages => "inline_images",
        SettingId::CodeLigatures => "code_ligatures",
        SettingId::Outline => "outline",
        SettingId::MenuBar => "menu_bar",
        SettingId::Spellcheck => "spellcheck",
        SettingId::WritingNits => "writing_nits",
        SettingId::FileVisibility => "file_visibility",
        SettingId::Autosave => "autosave",
        SettingId::LocalHistory => "history",
        SettingId::SessionRestore => "session_restore",
        SettingId::Keymap => "keymap",
        _ => return None,
    })
}

pub fn range_spec(id: SettingId) -> Option<&'static crate::range::RangeSpec> {
    Some(match id {
        SettingId::PageWidthProse => &crate::range::PAGE_WIDTH_PROSE,
        SettingId::PageWidthCode => &crate::range::PAGE_WIDTH_CODE,
        SettingId::Zoom => &crate::range::ZOOM,
        SettingId::ScrollSensitivity => &crate::range::SCROLL_SENSITIVITY,
        _ => return None,
    })
}

pub fn range_value(id: SettingId, values: &SettingsValues) -> Option<f32> {
    Some(match id {
        SettingId::PageWidthProse => values.page_width_prose as f32,
        SettingId::PageWidthCode => values.page_width_code as f32,
        SettingId::Zoom => values.zoom,
        SettingId::ScrollSensitivity => values.scroll_sensitivity,
        _ => return None,
    })
}

pub fn range_cell(row: &SettingRow, values: &SettingsValues) -> Option<crate::overlay::RangeCell> {
    let spec = range_spec(row.id)?;
    let v = range_value(row.id, values)?;
    Some(crate::overlay::RangeCell {
        id: row.id,
        step: spec.step_of(v),
    })
}

pub fn visible_range_cells(values: &SettingsValues) -> Vec<Option<crate::overlay::RangeCell>> {
    let cells: Vec<Option<crate::overlay::RangeCell>> = visible_rows()
        .iter()
        .map(|r| range_cell(r, values))
        .collect();
    if cells.iter().all(|c| c.is_none()) {
        return Vec::new();
    }
    cells
}

pub fn value_key(id: SettingId) -> Option<&'static str> {
    Some(match id {
        SettingId::PageWidthProse => "page_width_prose",
        SettingId::PageWidthCode => "page_width_code",
        SettingId::Zoom => "zoom",
        SettingId::ScrollSensitivity => "scroll_sensitivity",
        _ => return None,
    })
}

#[allow(dead_code)]
pub const PAGE_WIDTH_MIN: usize = crate::range::PAGE_WIDTH_PROSE.min as usize;
#[allow(dead_code)]
pub const PAGE_WIDTH_MAX: usize = crate::range::PAGE_WIDTH_PROSE.max as usize;

/// The config KEY a PATH row picks a folder for — the single owner of the
/// [`SettingId`] → config-key map for the folder-navigator route. `None` for a
/// non-path id. `App::setting_path_pick` writes this key (and for `project_root`
/// additionally re-scopes the active project). The RETURNED wire string is
/// UNCHANGED from before item 55 — see [`toggle_key`]'s doc.
pub fn path_key(id: SettingId) -> Option<&'static str> {
    Some(match id {
        SettingId::DefaultFolder => "default_folder",
        SettingId::ProjectsFolder => "workspace",
        SettingId::ProjectRoot => "project_root",
        _ => return None,
    })
}

/// Parse a typed ZOOM field into a clamped zoom FACTOR, or `None` if it isn't a
/// number. Accepts both the readout's own PERCENT form (`"80%"` → 0.8) and a bare
/// FACTOR (`"1.5"` → 1.5); an unsuffixed integer-ish value ≥ 10 is read as a
/// percent (`"125"` → 1.25) so retyping over the shown `"80%"` cell does the
/// obvious thing.
///
/// ITEM 94 — a one-line delegate to the ZOOM range spec's own
/// [`crate::range::RangeSpec::parse`] (which is where those accepted FORMS and the
/// 0.5..3.0 stepped clamp now live, shared with the rail and the readout). Kept as
/// a named door because the value-commit seam + its tests read it by name.
pub fn parse_zoom(raw: &str) -> Option<f32> {
    crate::range::ZOOM.parse(raw)
}

/// The SUB-PICKER a PICKER / SUBMENU row opens (Enter swaps the Settings overlay for
/// it, stamping a `return_to = Settings` breadcrumb so a commit/cancel returns here).
/// `None` for every non-picker id. The single owner of the [`SettingId`] → sub-overlay
/// map — the interaction reads it, never a parallel `match`.
pub fn sub_overlay(id: SettingId) -> Option<crate::overlay::OverlayKind> {
    Some(match id {
        SettingId::CaretStyle => crate::overlay::OverlayKind::Caret,
        SettingId::Theme => crate::overlay::OverlayKind::Theme,
        SettingId::Dictionary => crate::overlay::OverlayKind::Dictionary,
        SettingId::CjkReadsAs => crate::overlay::OverlayKind::CjkLang,
        SettingId::DateFormat => crate::overlay::OverlayKind::Date,
        SettingId::Keybindings => crate::overlay::OverlayKind::Keybindings,
        _ => return None,
    })
}

#[cfg(test)]
pub fn names() -> Vec<String> {
    SETTINGS.iter().map(|r| r.name.to_string()).collect()
}

#[cfg(test)]
pub fn value_cells(values: &SettingsValues) -> Vec<String> {
    SETTINGS.iter().map(|r| value_for(r, values)).collect()
}

// ── PLATFORM-SCOPED ROWS (RESOLVED — the web-config round) ─────────────────────
//
// "Edit config as text" used to hide on `Web`: `App::open_settings`
// (`app/files/`, the live handler `Effect::OpenSettings` reaches) early-returns
// on an empty `config.path`, and the web build used to hard-code `Config::empty()`
// (no `$XDG_CONFIG_HOME/awl/config.toml` in a browser sandbox — WEB.md's former
// "No config file on the web" gap). `main::wasm_start` now loads a real
// `config.toml` over `WebFs` (`fs::web_config_path`), so `config.path` is never
// empty there either — the row works identically on both platforms now, and
// `row_available_on` is kept as the one owner (rather than deleted outright) so a
// FUTURE platform-scoped row has a single door to extend, exactly like
// `commands::Command::available_on`.

/// Is `row` available on `platform`? Every row is available on every platform
/// today — kept as a real predicate (not inlined to `true`) so a future
/// platform-scoped Settings row has ONE owner to extend, mirroring
/// `commands::Command::available_on`.
fn row_available_on(_row: &SettingRow, _platform: crate::commands::Platform) -> bool {
    true
}

fn visible_rows_on(platform: crate::commands::Platform) -> Vec<&'static SettingRow> {
    SETTINGS
        .iter()
        .filter(|r| row_available_on(r, platform))
        .collect()
}

/// The catalog rows available on THIS COMPILED PLATFORM — the settings overlay's
/// ACTUAL corpus (built by `overlay::build`) and the view [`settings_accept`]
/// (`actions/overlay_nav.rs`) indexes back into, so a selected row index can never
/// mis-map once a row is hidden.
pub fn visible_rows() -> Vec<&'static SettingRow> {
    visible_rows_on(crate::commands::Platform::current())
}

pub static COVERED_BY: &[(SettingId, &str)] = &[
    (SettingId::Theme, "Switch theme…"),
    (SettingId::CaretStyle, "Caret style…"),
    (SettingId::Dictionary, "Dictionary…"),
    (SettingId::Keybindings, "Keybindings…"),
    (SettingId::ReportProblem, "Report a Problem"),
    (SettingId::PageMode, "Toggle page mode"),
    (SettingId::TypewriterScroll, "Toggle typewriter scroll"),
    (SettingId::Outline, "Toggle outline"),
    (SettingId::MenuBar, "Toggle menu bar"),
    (SettingId::Spellcheck, "Toggle spellcheck"),
    (SettingId::WritingNits, "Toggle writing nits"),
];

/// The covering command name for setting `id`, or `None` if it has no command
/// twin. Re-keyed onto [`SettingId`] (cheap hardening over the item-55 plan) so a
/// row RENAME can never silently drop a palette exclusion.
pub fn covered_by(id: SettingId) -> Option<&'static str> {
    COVERED_BY
        .iter()
        .find(|(row, _)| *row == id)
        .map(|(_, cmd)| *cmd)
}

/// The pure decision the palette filter rests on: is a row visible in the Cmd-P
/// palette union given its covering command name (`None` = uncovered) and
/// `platform`? Covered + the command is available there → HIDDEN (the command IS
/// the door); covered but the command is platform-hidden → VISIBLE (the door must
/// not be lost); uncovered → always visible. Exposed standalone (rather than
/// folded directly into [`palette_rows_on`]) so the platform-hidden REAPPEARANCE
/// behavior is directly testable against a hypothetical covering command, without
/// needing a real platform-scoped entry in [`COVERED_BY`] today (none of the ten
/// current covering commands are `native_only`/`web_only`).
pub fn row_visible_in_palette(covering: Option<&str>, platform: crate::commands::Platform) -> bool {
    match covering {
        Some(cmd) => !crate::commands::available_by_name(cmd, platform),
        None => true,
    }
}

fn palette_rows_on(platform: crate::commands::Platform) -> Vec<&'static SettingRow> {
    visible_rows_on(platform)
        .into_iter()
        .filter(|r| row_visible_in_palette(covered_by(r.id), platform))
        .collect()
}

pub fn palette_rows() -> Vec<&'static SettingRow> {
    palette_rows_on(crate::commands::Platform::current())
}

#[cfg(test)]
pub fn palette_names() -> Vec<String> {
    palette_rows().iter().map(|r| r.name.to_string()).collect()
}

pub fn palette_value_cells(values: &SettingsValues) -> Vec<String> {
    palette_rows()
        .iter()
        .map(|r| value_for(r, values))
        .collect()
}

pub fn visible_names() -> Vec<String> {
    visible_rows().iter().map(|r| r.name.to_string()).collect()
}

pub fn visible_value_cells(values: &SettingsValues) -> Vec<String> {
    visible_rows()
        .iter()
        .map(|r| value_for(r, values))
        .collect()
}

#[cfg(test)]
mod tests;
