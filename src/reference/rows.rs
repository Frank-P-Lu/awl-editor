//! THE ROW BUILDERS — every fact in the reference, read from the owner that
//! already holds it.
//!
//! Nothing in this file restates a roster. A command's name and chords come
//! from `commands::COMMANDS` (the same catalog the palette draws), a settings
//! row from `settings::SETTINGS`, a config key's default from the range spec /
//! toggle default that the running app itself reads, a world from
//! `theme::THEMES`, a markdown construct's coverage from `MdKind::tag`. What IS
//! authored here is the *presentation* — a column heading, a construct's
//! example line, a type's human name — and each authored list is pinned by a
//! law in `super::law` that fails when the roster it presents grows past it.

use super::Block;
use super::emit::{Cell, Table};

fn block(caption: Option<&str>, note: Option<&str>, table: Table) -> Block {
    Block {
        caption: caption.map(str::to_string),
        note: note.map(str::to_string),
        table,
    }
}

// ── COMMANDS ──────────────────────────────────────────────────────────────────

/// Every catalog command, grouped by the palette's own task taxonomy, with the
/// DEFAULT (config-free) chord it answers to under each convention.
///
/// Both columns are asked for EXPLICITLY (`Convention::Mac` / `Convention::Linux`,
/// `Platform::Native` throughout) rather than read from the running host, so the
/// document a macOS developer generates and the document Linux CI checks are the
/// same bytes. The Linux column's keep-list is `Config::empty()`'s — the default
/// composition — so a command left unbound on a stock Linux install shows an
/// empty cell rather than a chord no such install would honour.
pub(super) fn commands() -> Vec<Block> {
    let keep = crate::config::Config::empty().effective_linux_keep();
    let mut out = Vec::new();
    for cat in crate::commands::TaskCategory::ALL {
        let mut t = Table::new(&["Command", "macOS", "Linux", "Builds"]);
        for c in crate::commands::COMMANDS.iter() {
            if crate::commands::task_category_of(c.name) != Some(cat) {
                continue;
            }
            let mac = crate::commands::join_slots_truthful(
                c,
                crate::convention::Convention::Mac,
                crate::commands::Platform::Native,
                &[],
            );
            let linux = crate::commands::join_slots_truthful(
                c,
                crate::convention::Convention::Linux,
                crate::commands::Platform::Native,
                &keep,
            );
            t.push(vec![
                Cell::text(c.name),
                Cell::code_or_dash(&mac),
                Cell::code_or_dash(&linux),
                Cell::text(builds_label(c)),
            ]);
        }
        if !t.is_empty() {
            out.push(block(Some(cat.label()), None, t));
        }
    }
    out.push(synthetic_chords());
    out
}

/// The two chords the keymap matches DIRECTLY, outside the catalog: the command
/// palette (never a catalog row, so never rebindable through `[keys]`) and the
/// held stats HUD (a hold-only panel a discrete palette selection cannot
/// dismiss). Read from `keytoken::SYNTHETIC` — the same roster the docs' chord
/// tokens resolve through — so a third one cannot appear undocumented.
fn synthetic_chords() -> Block {
    let mut t = Table::new(&["Chord for", "macOS", "Linux"]);
    for (slug, _, _) in crate::keytoken::SYNTHETIC {
        let label = |c| {
            crate::keytoken::key_token_label(slug, c, crate::commands::Platform::Native)
                .unwrap_or_default()
        };
        t.push(vec![
            Cell::text(synthetic_name(slug)),
            Cell::code_or_dash(&label(crate::convention::Convention::Mac)),
            Cell::code_or_dash(&label(crate::convention::Convention::Linux)),
        ]);
    }
    block(
        Some("Chords with no command"),
        Some("These two are matched by the keymap directly and cannot be rebound."),
        t,
    )
}

/// The display name for a synthetic slug. `super::law` sweeps
/// `keytoken::SYNTHETIC` and fails by name for a slug with no entry here.
pub(super) fn synthetic_name(slug: &str) -> &'static str {
    match slug {
        "command_palette" => "Command palette",
        "stats_hud" => "Held stats HUD",
        other => panic!(
            "synthetic chord slug `{other}` has no display name in \
             `reference::rows::synthetic_name` — a chord the keymap matches \
             directly and the reference cannot name"
        ),
    }
}

/// Which builds carry a command. `native_only` and `web_only` are DATA on the
/// catalog row, so this is a readout, not a judgement.
fn builds_label(c: &crate::commands::Command) -> &'static str {
    match (c.native_only, c.web_only) {
        (false, false) => "Native, browser",
        (true, false) => "Native",
        (false, true) => "Browser",
        (true, true) => "None",
    }
}

// ── SETTINGS ──────────────────────────────────────────────────────────────────

/// Every row of the Settings overlay, in the order the overlay shows them, with
/// the `config.toml` key each one persists under.
pub(super) fn settings() -> Vec<Block> {
    let mut t = Table::new(&["Setting", "Group", "Control", "config.toml key"]);
    for r in crate::settings::SETTINGS {
        t.push(vec![
            Cell::text(r.name),
            Cell::text(r.category),
            Cell::text(control_label(r.kind)),
            Cell::code_or_dash(config_key_of(r.id).unwrap_or("")),
        ]);
    }
    vec![block(None, None, t)]
}

/// What the row's Enter key does. NO WILDCARD: a new `SettingKind` fails to
/// compile here until it says how it is edited.
fn control_label(k: crate::settings::SettingKind) -> &'static str {
    use crate::settings::SettingKind;
    match k {
        SettingKind::Toggle => "On/off",
        SettingKind::Picker => "Opens a picker",
        SettingKind::Value => "Typed value",
        SettingKind::Range => "Numeric rail",
        SettingKind::Path => "Picks a folder",
        SettingKind::Submenu => "Opens a submenu",
        SettingKind::Action => "Runs a command",
    }
}

/// A settings-row key that ROUTES rather than persists. `path_key` doubles as a
/// dispatch key: `App::setting_path_pick` intercepts `project_root` and performs
/// a switch-project instead of writing a pref, and `Config` has had no such field
/// since that key was retired. Documenting it as a `config.toml` key would tell a
/// reader to write a line the loader has never read.
///
/// `super::law::every_settings_row_key_is_a_real_config_key` pins this list from
/// both ends: no other row may name a key `Config` lacks, and an entry here that
/// no longer routes anything is stale.
pub(super) const SETTINGS_DISPATCH_ONLY_KEYS: &[&str] = &["project_root"];

/// The `config.toml` key a settings row writes, from the three no-wildcard maps
/// that already own that question, minus the dispatch-only keys above. A row
/// with no key persists nothing.
pub(super) fn config_key_of(id: crate::settings::SettingId) -> Option<&'static str> {
    let key = crate::settings::toggle_key(id)
        .or_else(|| crate::settings::value_key(id))
        .or_else(|| crate::settings::path_key(id))?;
    (!SETTINGS_DISPATCH_ONLY_KEYS.contains(&key)).then_some(key)
}

// ── CONFIGURATION FILE ────────────────────────────────────────────────────────

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
pub(super) fn documented_config_keys() -> Vec<&'static str> {
    CONFIG_KEYS.iter().map(|k| k.key).collect()
}

/// A `Config` field that is NOT a `config.toml` key: the loader records where it
/// read the file FROM. Named here so the destructuring law can account for every
/// field without documenting a key a user could never write.
pub(super) const CONFIG_NON_KEYS: &[&str] = &["path"];

/// What a key does when it is absent — read from the owner that supplies the
/// fallback, never from a doc comment. A boolean's default comes from the same
/// `Toggle` constant the running app initialises its process global with; a
/// numeric's from its `RangeSpec`; an enum's from that enum's own `Default`.
///
/// `menu_bar` is the one genuinely per-OS default. It is rendered from
/// `menubar`'s two authored consts rather than from `cfg!`, so this document is
/// the same bytes on macOS and on Linux CI.
pub(super) fn config_default(key: &str) -> Cell {
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
pub(super) fn config() -> Vec<Block> {
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

// ── WORLDS ────────────────────────────────────────────────────────────────────

/// Every theme world, its ground, and the two faces it wears.
pub(super) fn worlds() -> Vec<Block> {
    let mut t = Table::new(&["World", "Ground", "Display face", "Mono face"]);
    for th in crate::theme::THEMES.iter() {
        t.push(vec![
            Cell::text(th.name),
            Cell::text(if th.dark { "Dark" } else { "Light" }),
            Cell::text(th.font),
            Cell::text(th.mono),
        ]);
    }
    let note = format!(
        "The default world is {}. `--list-worlds` prints this roster; \
         `--theme <World>` selects one for a single run.",
        crate::theme::THEMES[crate::theme::DEFAULT_THEME].name
    );
    vec![block(None, Some(&note), t)]
}

// ── MARKDOWN ──────────────────────────────────────────────────────────────────

/// One markdown construct as a reader meets it: what it is called, how it is
/// written, and which `MdKind` span tags it accounts for.
///
/// `tags` is the drift anchor, not decoration:
/// `super::law::every_markdown_span_tag_is_documented` collects every tag
/// `MdKind::tag` can produce and fails by name for any tag no row here claims,
/// so a new span kind cannot ship undocumented. The `syntax` column is authored
/// — it is an example of the writing, not a fact about the tree.
struct Construct {
    name: &'static str,
    syntax: &'static str,
    tags: &'static [&'static str],
}

const CONSTRUCTS: &[Construct] = &[
    Construct {
        name: "Heading, levels 1–6",
        syntax: "# Heading",
        tags: &["h1", "h2", "h3", "h4", "h5", "h6"],
    },
    Construct {
        name: "Bold",
        syntax: "**bold**",
        tags: &["bold"],
    },
    Construct {
        name: "Italic",
        syntax: "*italic*",
        tags: &["italic"],
    },
    Construct {
        name: "Bold italic",
        syntax: "***both***",
        tags: &["bold_italic"],
    },
    Construct {
        name: "Inline code and code blocks",
        syntax: "`code`",
        tags: &["code"],
    },
    Construct {
        name: "Syntax highlighting in a fenced block",
        syntax: "```rust",
        tags: &[
            "code_comment",
            "code_comment_code",
            "code_string",
            "code_constant",
            "code_definition",
        ],
    },
    Construct {
        name: "Blockquote",
        syntax: "> quoted",
        tags: &["quote"],
    },
    Construct {
        name: "List, bulleted or numbered",
        syntax: "- item",
        tags: &["list_marker"],
    },
    Construct {
        name: "Link",
        syntax: "[text](target)",
        tags: &["link_text"],
    },
    Construct {
        name: "Task list",
        syntax: "- [ ] task",
        tags: &["task_open", "task_checked", "task_done"],
    },
    Construct {
        name: "Highlight",
        syntax: "==highlight==",
        tags: &["highlight"],
    },
    Construct {
        name: "Strikethrough",
        syntax: "~~struck~~",
        tags: &["strikethrough"],
    },
    Construct {
        name: "Thematic break",
        syntax: "---",
        tags: &["rule"],
    },
    Construct {
        name: "Table",
        syntax: "| a | b |",
        tags: &["table_pipe", "table_sep", "table_header"],
    },
    Construct {
        name: "Syntax characters of every construct above",
        syntax: "# * ` > [ ] |",
        tags: &["markup"],
    },
];

pub(super) fn documented_tags() -> Vec<&'static str> {
    CONSTRUCTS
        .iter()
        .flat_map(|c| c.tags.iter().copied())
        .collect()
}

/// Which markup hides while the caret is elsewhere. Every variant of
/// [`crate::markdown::ConcealKind`] appears, via a no-wildcard match — a new
/// conceal kind fails to compile until it declares its label and span.
pub(super) fn conceal_facts_for(
    k: crate::markdown::ConcealKind,
) -> (&'static str, &'static str, &'static str) {
    use crate::markdown::ConcealKind;
    match k {
        ConcealKind::Heading => ("Heading", "The leading `#` run", "The line"),
        ConcealKind::Emphasis => ("Bold and italic", "The `*` or `_` delimiters", "The line"),
        ConcealKind::Code => ("Inline code", "The backticks", "The line"),
        ConcealKind::Highlight => ("Highlight", "The `==` delimiters", "The line"),
        ConcealKind::Strikethrough => ("Strikethrough", "The `~~` delimiters", "The line"),
        ConcealKind::Fence => (
            "Fenced code block",
            "Both fence lines and the info string",
            "The whole block",
        ),
        ConcealKind::Frontmatter => ("Frontmatter", "The whole `---` block", "The whole block"),
        ConcealKind::Table => (
            "Table",
            "The whole source, replaced by a drawn grid",
            "The whole block",
        ),
        ConcealKind::Image => ("Image", "The whole `![alt](path)` source", "The line"),
        ConcealKind::Link => ("Link", "The brackets and the target", "The line"),
        ConcealKind::Blockquote => ("Blockquote", "The `>` marker", "The line"),
    }
}

pub(super) fn markdown() -> Vec<Block> {
    let mut constructs = Table::new(&["Construct", "Written as"]);
    for c in CONSTRUCTS {
        constructs.push(vec![Cell::text(c.name), Cell::code(c.syntax)]);
    }

    let mut conceal = Table::new(&[
        "Construct",
        "Hidden markup",
        "Revealed by",
        "Reveals in place",
    ]);
    for k in crate::markdown::ConcealKind::ALL {
        let (name, hidden, scope) = conceal_facts_for(k);
        conceal.push(vec![
            Cell::text(name),
            Cell::text(hidden),
            Cell::text(scope),
            Cell::text(if reveals_in_place(k) { "Yes" } else { "No" }),
        ]);
    }

    vec![
        block(
            Some("Constructs"),
            Some("The file stays plain text. Only the render changes."),
            constructs,
        ),
        block(
            Some("What hides off the caret"),
            Some(
                "With `wysiwyg = true`, the markup below hides while the caret \
                 and the selection are elsewhere.",
            ),
            conceal,
        ),
    ]
}

/// Asked of the renderer's OWN reveal rule rather than of a doc comment: does
/// putting the caret inside this construct's source un-conceal it where it
/// sits? A table answers no — its rows float over the drawn grid instead.
///
/// Both states are probed, not one. `conceal_off_cursor` is the caller's
/// already-computed "the caret's line is not this span's line", so a single
/// probe answers whichever question that flag was set to and reports it as if
/// it were the construct's property; asking for the caret INSIDE and OUTSIDE
/// and requiring the pair to differ is what actually measures a reveal.
fn reveals_in_place(k: crate::markdown::ConcealKind) -> bool {
    let span = 10..20;
    let inside = crate::render::wysiwyg_reveals(k, false, 12, &span, None);
    let outside = crate::render::wysiwyg_reveals(k, true, 40, &span, None);
    inside && !outside
}
