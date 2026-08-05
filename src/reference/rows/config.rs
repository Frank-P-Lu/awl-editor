//! The configuration section's rows: every `config.toml` key, the shape of its
//! value, and what it is when absent — each default read from the owner that
//! supplies the fallback rather than from a doc comment.

use super::super::Block;
use super::super::emit::{Cell, Table};
use super::block;

/// One documented `config.toml` key. `key` must name a real field of
/// [`crate::config::Config`] — `super::law::every_config_field_is_documented`
/// destructures that struct with no `..` arm, so a new field fails to COMPILE
/// there until it is either listed here or explicitly named as unreachable from
/// the file.
struct ConfigKey {
    key: &'static str,
    ty: ConfigType,
}

/// The shape a key's value takes. `Choice` renders its alternatives from the
/// enum roster that parses them, so a new caret mode / dictionary / date format
/// / keymap flavour widens the documented type with no edit here.
enum ConfigType {
    Bool,
    Path,
    World,
    Percent,
    Columns,
    Choice(fn() -> String),
    List(&'static str),
    KeyTable,
}

impl ConfigType {
    fn label(&self) -> String {
        match self {
            ConfigType::Bool => "true | false".to_string(),
            ConfigType::Path => "path".to_string(),
            ConfigType::World => "world name".to_string(),
            ConfigType::Percent => "percent".to_string(),
            ConfigType::Columns => "whole columns".to_string(),
            ConfigType::Choice(f) => f(),
            ConfigType::List(what) => format!("list of {what}"),
            ConfigType::KeyTable => "table of chord lists".to_string(),
        }
    }
}

fn caret_modes() -> String {
    crate::caret::CaretMode::ALL
        .iter()
        .map(|m| crate::config::caret_mode_name(*m))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn dictionaries() -> String {
    crate::spell::DictVariant::ALL
        .iter()
        .map(|d| crate::config::dictionary_name(*d))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn date_formats() -> String {
    crate::dateformat::DateFormat::ALL
        .iter()
        .map(|d| d.config_name())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn keymap_flavors() -> String {
    crate::keymap::KeymapFlavor::ALL
        .iter()
        .map(|f| f.config_name())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Every key `Config::load` reads out of `config.toml`, in the order the parser
/// reaches them.
const CONFIG_KEYS: &[ConfigKey] = &[
    ConfigKey {
        key: "default_folder",
        ty: ConfigType::Path,
    },
    ConfigKey {
        key: "workspace",
        ty: ConfigType::Path,
    },
    ConfigKey {
        key: "theme",
        ty: ConfigType::World,
    },
    ConfigKey {
        key: "zoom",
        ty: ConfigType::Percent,
    },
    ConfigKey {
        key: "scroll_sensitivity",
        ty: ConfigType::Percent,
    },
    ConfigKey {
        key: "page_mode",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "page_width_prose",
        ty: ConfigType::Columns,
    },
    ConfigKey {
        key: "page_width_code",
        ty: ConfigType::Columns,
    },
    ConfigKey {
        key: "caret_mode",
        ty: ConfigType::Choice(caret_modes),
    },
    ConfigKey {
        key: "dictionary",
        ty: ConfigType::Choice(dictionaries),
    },
    ConfigKey {
        key: "writing_nits",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "spellcheck",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "history",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "autosave",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "wysiwyg",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "popover",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "inline_images",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "code_ligatures",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "cjk_priority",
        ty: ConfigType::List("language codes"),
    },
    ConfigKey {
        key: "session_restore",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "outline",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "menu_bar",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "typewriter_scroll",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "file_visibility",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "stats",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "reduce_motion",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "ambient_motion",
        ty: ConfigType::Bool,
    },
    ConfigKey {
        key: "keymap",
        ty: ConfigType::Choice(keymap_flavors),
    },
    ConfigKey {
        key: "date_format",
        ty: ConfigType::Choice(date_formats),
    },
    ConfigKey {
        key: "keys",
        ty: ConfigType::KeyTable,
    },
    ConfigKey {
        key: "linux_keep_emacs",
        ty: ConfigType::List("chords"),
    },
];

/// Every key the reference documents — the roster half of
/// `super::law::every_config_field_is_documented`.
pub(crate) fn documented_config_keys() -> Vec<&'static str> {
    CONFIG_KEYS.iter().map(|k| k.key).collect()
}

/// A `Config` field that is NOT a `config.toml` key: the loader records where it
/// read the file FROM. Named here so the destructuring law can account for every
/// field without documenting a key a user could never write.
pub(crate) const CONFIG_NON_KEYS: &[&str] = &["path"];

/// What a key does when it is absent — read from the owner that supplies the
/// fallback, never from a doc comment. A boolean's default comes from the same
/// `Toggle` constant the running app initialises its process global with; a
/// numeric's from its `RangeSpec`; an enum's from that enum's own `Default`.
///
/// `menu_bar` is the one genuinely per-OS default. It is rendered from
/// `menubar`'s two authored consts rather than from `cfg!`, so this document is
/// the same bytes on macOS and on Linux CI.
pub(crate) fn config_default(key: &str) -> Cell {
    let empty = crate::config::Config::empty();
    match key {
        "default_folder" | "workspace" | "cjk_priority" | "keys" | "linux_keep_emacs" => Cell::Dash,
        "theme" => Cell::code(crate::theme::THEMES[crate::theme::DEFAULT_THEME].name),
        "zoom" => range_default(&crate::range::ZOOM),
        "scroll_sensitivity" => range_default(&crate::range::SCROLL_SENSITIVITY),
        "page_width_prose" => range_default(&crate::range::PAGE_WIDTH_PROSE),
        "page_width_code" => range_default(&crate::range::PAGE_WIDTH_CODE),
        "caret_mode" => Cell::code(crate::config::caret_mode_name(crate::caret::default_mode())),
        "dictionary" => Cell::code(crate::config::dictionary_name(
            crate::spell::DictVariant::DEFAULT,
        )),
        "keymap" => Cell::code(crate::keymap::KeymapFlavor::default().config_name()),
        "date_format" => Cell::code(crate::dateformat::DateFormat::default().config_name()),
        "menu_bar" => Cell::text(format!(
            "{} on macOS, {} elsewhere",
            bool_word(crate::menubar::MENU_BAR_DEFAULT_MACOS),
            bool_word(crate::menubar::MENU_BAR_DEFAULT_OTHER)
        )),
        "autosave" => Cell::code(bool_word(empty.autosave_on())),
        "history" => Cell::code(bool_word(empty.history_on())),
        "session_restore" => Cell::code(bool_word(empty.session_restore_on())),
        "ambient_motion" => Cell::code(bool_word(empty.ambient_motion_on())),
        "stats" => Cell::code(bool_word(empty.stats_on())),
        other => match crate::settings::toggle_default(other) {
            Some(on) => Cell::code(bool_word(on)),
            None => panic!(
                "config key `{other}` has no default owner — add it to \
                 `reference::rows::config_default` reading the owner that \
                 actually supplies its fallback, never a literal"
            ),
        },
    }
}

fn bool_word(on: bool) -> &'static str {
    if on { "true" } else { "false" }
}

/// A range's authored value, formatted by its UNIT rather than by
/// `RangeSpec::format` — which is the READOUT formatter and clamps into the
/// band first. Asked for a step of 0.1 on a band starting at 0.5, it answers
/// "50%", so the reference's Step column silently printed each band's minimum
/// until this spot-check caught it.
fn spec_value(spec: &crate::range::RangeSpec, v: f32) -> Cell {
    Cell::code(spec.unit.format(v))
}

fn range_default(spec: &crate::range::RangeSpec) -> Cell {
    spec_value(spec, spec.default)
}

/// The `config.toml` reference: every key, its accepted value, and what it is
/// when absent — plus the four numeric keys' authored bands.
pub(crate) fn config() -> Vec<Block> {
    let mut keys = Table::new(&["Key", "Value", "Default"]);
    for k in CONFIG_KEYS {
        keys.push(vec![
            Cell::code(k.key),
            Cell::text(k.ty.label()),
            config_default(k.key),
        ]);
    }

    let mut ranges = Table::new(&["Key", "Minimum", "Maximum", "Step", "Default"]);
    for (key, spec) in [
        ("zoom", &crate::range::ZOOM),
        ("scroll_sensitivity", &crate::range::SCROLL_SENSITIVITY),
        ("page_width_prose", &crate::range::PAGE_WIDTH_PROSE),
        ("page_width_code", &crate::range::PAGE_WIDTH_CODE),
    ] {
        ranges.push(vec![
            Cell::code(key),
            spec_value(spec, spec.min),
            spec_value(spec, spec.max),
            spec_value(spec, spec.step),
            spec_value(spec, spec.default),
        ]);
    }

    vec![
        block(
            Some("Keys"),
            Some(
                "An absent key takes the default below. A command-line flag \
                 overrides the file; the file overrides the default.",
            ),
            keys,
        ),
        block(
            Some("Numeric bands"),
            Some("A value outside the band is clamped to it, then snapped to the step."),
            ranges,
        ),
    ]
}
