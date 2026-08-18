use std::path::{Path, PathBuf};

pub struct Config {
    pub default_folder: Option<PathBuf>,
    pub workspace: Option<PathBuf>,
    pub theme: Option<String>,
    pub zoom: Option<f32>,
    pub scroll_sensitivity: Option<f32>,
    pub page_mode: Option<bool>,
    pub page_width_prose: Option<usize>,
    pub page_width_code: Option<usize>,
    pub caret_mode: Option<String>,
    pub dictionary: Option<String>,
    pub writing_nits: Option<bool>,
    pub spellcheck: Option<bool>,
    pub history: Option<bool>,
    pub autosave: Option<bool>,
    pub wysiwyg: Option<bool>,
    pub popover: Option<bool>,
    pub inline_images: Option<bool>,
    pub code_ligatures: Option<bool>,
    pub cjk_priority: Option<Vec<crate::frontmatter::Lang>>,
    pub session_restore: Option<bool>,
    pub outline: Option<bool>,
    pub menu_bar: Option<bool>,
    pub typewriter_scroll: Option<bool>,
    pub file_visibility: Option<bool>,
    pub stats: Option<bool>,
    pub reduce_motion: Option<bool>,
    pub ambient_motion: Option<bool>,
    pub keymap: Option<String>,
    pub date_format: Option<String>,
    pub keys: Vec<(String, Vec<String>)>,
    pub linux_keep_emacs: Vec<String>,
    pub path: PathBuf,
}

impl Config {
    pub fn empty() -> Self {
        Config {
            default_folder: None,
            workspace: None,
            theme: None,
            zoom: None,
            scroll_sensitivity: None,
            page_mode: None,
            page_width_prose: None,
            page_width_code: None,
            caret_mode: None,
            dictionary: None,
            writing_nits: None,
            spellcheck: None,
            history: None,
            autosave: None,
            wysiwyg: None,
            popover: None,
            inline_images: None,
            code_ligatures: None,
            cjk_priority: None,
            session_restore: None,
            outline: None,
            menu_bar: None,
            typewriter_scroll: None,
            file_visibility: None,
            stats: None,
            reduce_motion: None,
            ambient_motion: None,
            keymap: None,
            date_format: None,
            keys: Vec::new(),
            linux_keep_emacs: Vec::new(),
            path: PathBuf::new(),
        }
    }

    pub fn history_on(&self) -> bool {
        self.history.unwrap_or(true)
    }

    pub fn autosave_on(&self) -> bool {
        self.autosave.unwrap_or(true)
    }

    pub fn session_restore_on(&self) -> bool {
        self.session_restore.unwrap_or(true)
    }

    pub fn ambient_motion_on(&self) -> bool {
        self.ambient_motion.unwrap_or(true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn stats_on(&self) -> bool {
        self.stats.unwrap_or(true)
    }

    #[cfg(test)]
    pub fn outline_on(&self) -> bool {
        self.outline.unwrap_or(true)
    }

    #[cfg(test)]
    pub fn menu_bar_on(&self) -> bool {
        self.menu_bar.unwrap_or(crate::menubar::menu_bar_default())
    }

    pub fn keymap_flavor(&self) -> crate::keymap::KeymapFlavor {
        self.keymap
            .as_deref()
            .and_then(crate::keymap::KeymapFlavor::parse)
            .unwrap_or_default()
    }

    /// THE ONE COMPOSITION OWNER of the effective Linux keep-list — every
    /// caller (dispatch, label truth, the rebind menu, this struct's own
    /// callers) reads the union this function returns, never re-derives it.
    /// The native-clipboard carve-out lives HERE and nowhere else: the
    /// `emacs` flavor preset's own contribution skips `"C-c"`/`"C-v"`
    /// (`linux_is_native_clipboard_chord`) so Copy/Paste stay native under
    /// `keymap = "emacs"` — a compositor that forwards Super+C/V as Ctrl+C/V
    /// (Omarchy/Hyprland) needs no extra config. `"C-x"` is untouched (still
    /// widened by the preset), so it stays the emacs prefix. The user's own
    /// explicit `linux_keep_emacs` entries below are NOT filtered — naming
    /// `"C-c"`/`"C-v"` there is a deliberate per-chord ask, unaffected by the
    /// preset's own carve-out.
    pub fn effective_linux_keep(&self) -> Vec<String> {
        let mut keep: Vec<String> = crate::keymap::linux_builtin_keep()
            .iter()
            .map(|s| s.to_string())
            .collect();
        if self.keymap_flavor() == crate::keymap::KeymapFlavor::Emacs {
            for p in crate::keymap::linux_emacs_preset_keep() {
                if crate::keymap::linux_is_native_clipboard_chord(&p) {
                    continue;
                }
                if !crate::keymap::linux_keeps_chord(&keep, &p) {
                    keep.push(p);
                }
            }
        }
        for k in &self.linux_keep_emacs {
            if !crate::keymap::linux_keeps_chord(&keep, k) {
                keep.push(k.clone());
            }
        }
        keep
    }

    pub fn cjk_priority_or_default(&self) -> Vec<crate::frontmatter::Lang> {
        match &self.cjk_priority {
            Some(v) if !v.is_empty() => v.clone(),
            _ => crate::frontmatter::DEFAULT_CJK_PRIORITY.to_vec(),
        }
    }

    pub fn measure_for(&self, class: crate::page::PageClass) -> usize {
        let configured = match class {
            crate::page::PageClass::Prose => self.page_width_prose,
            crate::page::PageClass::Code => self.page_width_code,
        };
        configured.unwrap_or_else(|| class.default_measure())
    }

    pub fn load(path: PathBuf) -> Self {
        let mut cfg = Self::empty();
        cfg.path = path;
        let src = match crate::fs::active().read_to_string(&cfg.path) {
            Ok(s) => s,
            Err(_) => return cfg, // absent/unreadable: pure defaults, no behaviour change
        };
        let table: toml::Table = match src.parse() {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "config {}: parse error: {e}; using defaults",
                    cfg.path.display()
                );
                return cfg;
            }
        };
        if let Some(s) = table.get("default_folder").and_then(|v| v.as_str()) {
            cfg.default_folder = Some(expand_tilde(s));
        }
        if let Some(s) = table.get("workspace").and_then(|v| v.as_str()) {
            cfg.workspace = Some(expand_tilde(s));
        }
        if let Some(s) = table.get("theme").and_then(|v| v.as_str()) {
            cfg.theme = Some(s.to_string());
        }
        (cfg.zoom, cfg.scroll_sensitivity) = super::sticky::numeric_ranges(&table);
        debug_assert!(
            cfg.scroll_sensitivity.is_none() || cfg.scroll_sensitivity.unwrap().is_finite()
        );
        cfg.page_mode = table.get("page_mode").and_then(|v| v.as_bool());
        if let Some(w) = table.get("page_width_prose").and_then(toml_as_usize) {
            cfg.page_width_prose = Some(w.max(1));
        }
        if let Some(w) = table.get("page_width_code").and_then(toml_as_usize) {
            cfg.page_width_code = Some(w.max(1));
        }
        if let Some(s) = table.get("caret_mode").and_then(|v| v.as_str()) {
            cfg.caret_mode = Some(s.to_string());
        }
        if let Some(s) = table.get("dictionary").and_then(|v| v.as_str()) {
            cfg.dictionary = Some(s.to_string());
        }
        apply_boolean_settings(&mut cfg, &table);
        if let Some(arr) = table.get("cjk_priority").and_then(|v| v.as_array()) {
            let langs: Vec<crate::frontmatter::Lang> = arr
                .iter()
                .filter_map(|v| v.as_str().and_then(crate::frontmatter::Lang::parse))
                .collect();
            cfg.cjk_priority = Some(langs);
        }
        if let Some(s) = table.get("keymap").and_then(|v| v.as_str()) {
            cfg.keymap = Some(s.to_string());
        }
        if let Some(s) = table.get("date_format").and_then(|v| v.as_str()) {
            cfg.date_format = Some(s.to_string());
        }
        if let Some(arr) = table.get("linux_keep_emacs").and_then(|v| v.as_array()) {
            cfg.linux_keep_emacs = arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect();
        }
        if let Some(keys) = table.get("keys").and_then(|v| v.as_table()) {
            for (name, val) in keys {
                let chords: Vec<String> = match val {
                    toml::Value::String(s) => vec![s.clone()],
                    toml::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .take(2)
                        .collect(),
                    _ => continue,
                };
                if !chords.is_empty() {
                    cfg.keys.push((name.clone(), chords));
                }
            }
        }
        cfg
    }
}

fn apply_boolean_settings(cfg: &mut Config, table: &toml::Table) {
    let value = |key| table.get(key).and_then(toml::Value::as_bool);
    cfg.writing_nits = value("writing_nits");
    cfg.spellcheck = value("spellcheck");
    cfg.history = value("history");
    cfg.autosave = value("autosave");
    cfg.popover = value("popover");
    cfg.wysiwyg = value("wysiwyg");
    cfg.inline_images = value("inline_images");
    cfg.code_ligatures = value("code_ligatures");
    cfg.session_restore = value("session_restore");
    cfg.outline = value("outline");
    cfg.menu_bar = value("menu_bar");
    cfg.typewriter_scroll = value("typewriter_scroll");
    cfg.stats = value("stats");
    cfg.file_visibility = value("file_visibility");
    cfg.reduce_motion = value("reduce_motion");
    cfg.ambient_motion = value("ambient_motion");
}

pub fn caret_mode_name(m: crate::caret::CaretMode) -> &'static str {
    match m {
        crate::caret::CaretMode::Block => "block",
        crate::caret::CaretMode::Morph => "morph",
        crate::caret::CaretMode::Ibeam => "ibeam",
    }
}

pub fn parse_caret_mode(s: &str) -> Option<crate::caret::CaretMode> {
    match s.trim().to_ascii_lowercase().as_str() {
        "block" => Some(crate::caret::CaretMode::Block),
        "morph" => Some(crate::caret::CaretMode::Morph),
        "ibeam" => Some(crate::caret::CaretMode::Ibeam),
        _ => None,
    }
}

pub fn dictionary_name(v: crate::spell::DictVariant) -> &'static str {
    match v {
        crate::spell::DictVariant::EnUs => "en_US",
        crate::spell::DictVariant::EnGb => "en_GB",
        crate::spell::DictVariant::EnAu => "en_AU",
    }
}

pub fn parse_dictionary(s: &str) -> Option<crate::spell::DictVariant> {
    match s.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "en_us" => Some(crate::spell::DictVariant::EnUs),
        "en_gb" => Some(crate::spell::DictVariant::EnGb),
        "en_au" => Some(crate::spell::DictVariant::EnAu),
        _ => None,
    }
}

pub fn config_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(p) = explicit {
        return p;
    }
    if let Some(p) = std::env::var_os("AWL_CONFIG") {
        return PathBuf::from(p);
    }
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(x).join("awl").join("config.toml");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("awl")
            .join("config.toml");
    }
    PathBuf::from("awl-config.toml")
}

pub fn dictionary_path(config_path: &Path) -> Option<PathBuf> {
    let parent = config_path.parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    Some(parent.join("dictionary.txt"))
}

pub(super) fn toml_as_f32(v: &toml::Value) -> Option<f32> {
    v.as_float()
        .map(|f| f as f32)
        .or_else(|| v.as_integer().map(|i| i as f32))
        .filter(|f| f.is_finite())
}

fn toml_as_usize(v: &toml::Value) -> Option<usize> {
    v.as_integer()
        .and_then(|i| usize::try_from(i).ok())
        .or_else(|| {
            v.as_float()
                .filter(|f| *f >= 0.0)
                .map(|f| f.round() as usize)
        })
}

pub(super) fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(s)
}
