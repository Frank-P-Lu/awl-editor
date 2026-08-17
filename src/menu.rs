//! Shared menu roster and routing.
//! Every routed item resolves to an existing catalog action. Native macOS and
//! awl-drawn menu bars consume the same data; native construction lives in
//! [`native`]. Quit stays routed through awl's clean-shutdown path.
//!
use crate::commands;
use crate::keymap::Action;

#[cfg(target_os = "macos")]
mod native;
#[cfg(target_os = "macos")]
pub use native::{InstalledMenu, install};
#[cfg(test)]
mod chord_truth;
#[cfg(test)]
mod ellipsis_law;

/// Context policy shared by the native updater and the roster laws. This stays
/// platform-neutral so the state transition can be proved without constructing
/// AppKit objects in a test process.
pub fn markdown_submenu_enabled(is_markdown: bool) -> bool {
    is_markdown
}

/// One routed menu item and its catalog command.
struct Routed {
    id: &'static str,
    command: &'static str,
    label: &'static str,
    icon: bool,
}

/// Build the common text-only item whose label matches its command name.
const fn r(id: &'static str, command: &'static str) -> Routed {
    Routed {
        id,
        command,
        label: command,
        icon: false,
    }
}

const fn ri(id: &'static str, command: &'static str) -> Routed {
    Routed {
        id,
        command,
        label: command,
        icon: true,
    }
}

/// App menu's THREE routed items — About (an in-app card, see `about.rs`),
/// Settings (P1 of the keybinding-idiom audit — Cmd-, is the preferences
/// chord since Mac OS X 10.1), and Quit (see the module doc for why all three
/// are routed rather than muda's predefined items). About's and Quit's labels
/// append "Awl" per the stock macOS App-menu convention (every system app's
/// About/Quit items name the app); Settings keeps its bare catalog name
/// ("Settings…" is already unambiguous). All three CATALOG names ("About" /
/// "Settings…" / "Quit") stay what the Cmd-P palette shows.
const APP_ITEMS: &[Routed] = &[
    Routed {
        id: "awl.about",
        command: "About",
        label: "About Awl",
        icon: false,
    },
    Routed {
        id: "awl.settings",
        command: "Settings…",
        label: "Settings…",
        icon: false,
    },
    Routed {
        id: "awl.quit",
        command: "Quit",
        label: "Quit Awl",
        icon: false,
    },
];

const FILE_ITEMS: &[Routed] = &[
    ri("awl.new_document", "New document"),
    r("awl.command_palette", "Command palette…"),
    r("awl.goto", "Go to…"),
    ri("awl.open", "Open file…"),
    Routed {
        id: "awl.open_folder",
        command: "Open folder…",
        label: "Open folder…",
        icon: true,
    },
    // "Recent projects" is a SINGLE File item that opens the SWITCH-PROJECT
    // navigator pre-lensed onto its Recent lens (`Action::OpenRecentProjects` — the
    // fold that retired the standalone RecentProjects picker), not a dynamic
    // Open-Recent SUBMENU of the roots themselves — a deliberate scope choice: this
    // menu bar is PURE STATIC DATA routed by an id → catalog-command-NAME table
    // ([`SECTIONS`]), and each recent root is runtime state, not a catalog command,
    // so it has no place in that table. The navigator (fuzzy-filterable,
    // keyboard-drivable, shared with the palette command) is the one door; a live
    // submenu is a possible future round. No icon (kept minimal, like most items).
    Routed {
        id: "awl.rename_file",
        command: "Rename note…",
        label: "Rename file…",
        icon: false,
    },
    Routed {
        id: "awl.move_file",
        command: "Move…",
        label: "Move file…",
        icon: false,
    },
    Routed {
        id: "awl.duplicate_file",
        command: "Duplicate note",
        label: "Duplicate file",
        icon: false,
    },
    r("awl.history", "Version history…"),
    ri("awl.save", "Save"),
    Routed {
        id: "awl.finish_buffer",
        command: "Finish file",
        label: "Save and return",
        icon: true,
    },
    // THE THREE EXPORT ROWS CARRY THE ELLIPSIS AGAIN, and `menu::ellipsis_law`
    // is what re-earned it: an export now asks WHERE before it writes — the
    // destination navigator on every platform's shared core, and `NSSavePanel`
    // from this menu on macOS. See `ellipsis_law`'s module doc for the platform
    // the one static string still over-promises to.
    r("awl.export_pdf", "Export as PDF…"),
    r("awl.export_word", "Export as Word…"),
    r("awl.export_html", "Export as HTML…"),
];

/// The menu ids native macOS answers with a REAL AppKit PANEL instead of
/// dispatching the routed action — the platform's own convention winning over
/// awl's in-app overlay for that one verb. Declared HERE, as data beside the
/// roster it indexes, rather than as a literal inside the macOS event handler:
/// the panel is a second door onto the same promise a label's ellipsis makes,
/// and `menu::ellipsis_law` can only hold both doors to that promise if both
/// are readable from any host. `crate::app::menu`'s handler is the one
/// dispatcher; `opens_native_panel` is the one predicate.
///
/// Only the MENU is redirected. The keyboard chord and the Cmd-P palette row
/// for the same command keep the in-app overlay on every platform, so the
/// shared core's own behaviour is what a label has to agree with.
const NATIVE_PANEL_IDS: &[&str] = &[
    // File ▸ "Browse files…" → `NSOpenPanel`.
    "awl.open",
    // File ▸ the three "Export as …" rows → `NSSavePanel`, opened at the folder
    // and under the name the destination owner would have chosen on its own
    // (`app::files::export::export_target`). The shared core's own answer for the
    // same commands — the `ExportDest` navigator — is what the keyboard and the
    // palette still reach, and what the drawn menu bar reaches on Linux and web.
    "awl.export_pdf",
    "awl.export_word",
    "awl.export_html",
];

/// Whether the native macOS menu answers `id` with an AppKit panel of its own.
// Read by the macOS menu handler and by `ellipsis_law`; a Linux/web build routes
// nothing through it, and the roster law still has to be able to ask.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub fn opens_native_panel(id: &str) -> bool {
    NATIVE_PANEL_IDS.contains(&id)
}

const EDIT_ITEMS: &[Routed] = &[
    r("awl.undo", "Undo"),
    r("awl.redo", "Redo"),
    r("awl.cut", "Cut"),
    r("awl.copy", "Copy"),
    r("awl.paste", "Paste"),
    r("awl.select_all", "Select all"),
];

/// The Markdown child menu is deliberately stable: its complete writing
/// vocabulary stays visible as one menu on every platform, and the surfaces
/// disable it as a whole for a non-Markdown buffer rather than making the
/// surrounding Edit menu jump as files change.
const MARKDOWN_ITEMS: &[Routed] = &[
    r("awl.bold", "Bold"),
    r("awl.italic", "Italic"),
    r("awl.inline_code", "Inline code"),
    r("awl.highlight", "Highlight"),
    r("awl.strikethrough", "Strikethrough"),
    r("awl.insert_link", "Insert link…"),
    r("awl.heading", "Heading"),
    r("awl.blockquote", "Blockquote"),
    r("awl.bullet_list", "Bullet list"),
    r("awl.numbered_list", "Numbered list"),
    r("awl.task_list", "Task list"),
    r("awl.code_block", "Code block"),
    r("awl.align_table", "Align table"),
];

const VIEW_ITEMS: &[Routed] = &[
    r("awl.toggle_page_mode", "Toggle page mode"),
    ri("awl.switch_theme", "Switch theme…"),
    r("awl.zoom_in", "Zoom in"),
    r("awl.zoom_out", "Zoom out"),
    r("awl.reset_zoom", "Reset zoom"),
    r("awl.toggle_debug", "Toggle debug"),
    r("awl.narrow_page", "Narrow page"),
    r("awl.widen_page", "Widen page"),
    r("awl.reset_page_width", "Reset page width"),
    Routed {
        id: "awl.page_width_settings",
        command: "Settings…",
        label: "Page width settings…",
        icon: false,
    },
    r("awl.toggle_outline", "Toggle outline"),
    r("awl.fold_section", "Fold section"),
    r("awl.collapse_other_sections", "Collapse other sections"),
    r("awl.toggle_typewriter", "Toggle typewriter scroll"),
    r("awl.toggle_menu_bar", "Toggle menu bar"),
];

const HELP_ITEMS: &[Routed] = &[
    r("awl.guide", "Guide"),
    r("awl.credits", "Credits"),
    r("awl.check_for_updates", "Check for Updates"),
    r("awl.report_problem", "Report a Problem"),
];

const SECTIONS: &[&[Routed]] = &[
    APP_ITEMS,
    FILE_ITEMS,
    EDIT_ITEMS,
    MARKDOWN_ITEMS,
    VIEW_ITEMS,
    HELP_ITEMS,
];

/// A muda PREDEFINED item this menu bar uses — no `Action`, no catalog entry:
/// genuinely OS chrome (a window-manager command), never app behavior (see
/// the module doc's Quit/About-vs-predefined decisions for why that boundary
/// is drawn here and not wider — both are now routed instead).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredefinedKind {
    Minimize,
    Maximize,
    /// App-menu Hide (⌘H, macOS gives the accelerator for free). P3 of the
    /// keybinding-idiom audit.
    Hide,
    HideOthers,
    ShowAll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RosterItem {
    Routed {
        id: &'static str,
        label: &'static str,
        icon: bool,
    },
    Predefined(PredefinedKind),
    Submenu {
        label: &'static str,
        items: Vec<RosterItem>,
    },
    Separator,
}

#[derive(Debug, PartialEq)]
pub struct RosterMenu {
    pub title: &'static str,
    pub items: Vec<RosterItem>,
}

fn routed(item: &Routed) -> RosterItem {
    RosterItem::Routed {
        id: item.id,
        label: item.label,
        icon: item.icon,
    }
}

/// The menu bar structure for THIS COMPILED PLATFORM (`commands::Platform::current()`)
/// — pure data, ZERO muda calls, so it is buildable and assertable from any thread
/// (see the module doc for why [`build_menu`], unlike this, is live-only).
/// [`build_menu`] translates this EXACT data into real muda types, so the built menu
/// can never diverge from what this function (and its tests) describe. On native this
/// is BYTE-IDENTICAL to the full roster [`roster_all`] describes (nothing is hidden);
/// on web it is [`roster_all`] filtered through [`roster_for`] — see that function's
/// doc for what drops and why.
pub fn roster() -> Vec<RosterMenu> {
    roster_for(commands::Platform::current())
}

/// [`roster`], parameterized by an EXPLICIT platform — the seam that lets a native-run
/// test assert the WEB-filtered roster (`roster_for(Platform::Web)`) without any `cfg!`
/// gymnastics or an actual wasm build. PLATFORM-SCOPED COMMANDS: a ROUTED item whose
/// catalog action is unavailable on `platform` (`commands::action_available`) is
/// dropped; a PREDEFINED item (genuine OS window-manager chrome — Minimize/Zoom) is
/// dropped outright on `Platform::Web` (there is no OS window to minimize/zoom in a
/// browser tab) and kept on `Platform::Native` (every native platform, including
/// Linux, where the awl-rendered bar still shows them as the existing inert dead
/// rows — unchanged v1 behavior, only web newly prunes them). Any separator left
/// dangling by a drop (leading, trailing, or doubled-up) is trimmed so the visible
/// list never opens or closes on a rule. A menu left with ZERO items after filtering
/// is dropped entirely — this is what removes the whole Window menu on web (both its
/// items are predefined) and the "Quit Awl" + the predefined Hide block (both
/// dropped on web — Quit is `native_only`, Hide/Hide Others/Show All are OS window
/// chrome) plus their now-dangling separators from the App menu, leaving "About Awl"
/// and "Settings…" (neither `native_only`) with exactly one separator between them.
pub fn roster_for(platform: commands::Platform) -> Vec<RosterMenu> {
    roster_all()
        .into_iter()
        .map(|m| RosterMenu {
            title: m.title,
            items: filter_items_for_platform(m.items, platform),
        })
        .filter(|m| !m.items.is_empty())
        .collect()
}

fn filter_items_for_platform(
    items: Vec<RosterItem>,
    platform: commands::Platform,
) -> Vec<RosterItem> {
    let kept: Vec<RosterItem> = items
        .into_iter()
        .filter_map(|item| match item {
            RosterItem::Routed { id, label, icon } => resolve(id)
                .filter(|action| commands::action_available(action, platform))
                .map(|_| RosterItem::Routed { id, label, icon }),
            RosterItem::Predefined(kind) => {
                (platform == commands::Platform::Native).then_some(RosterItem::Predefined(kind))
            }
            RosterItem::Submenu { label, items } => {
                let items = filter_items_for_platform(items, platform);
                (!items.is_empty()).then_some(RosterItem::Submenu { label, items })
            }
            RosterItem::Separator => Some(RosterItem::Separator), // dangling ones trimmed below
        })
        .collect();
    trim_separators(kept)
}

/// Drop a LEADING separator, collapse consecutive separators to one, and drop a
/// TRAILING separator — so a menu whose surrounding items got filtered away never
/// opens or closes on a bare rule.
fn trim_separators(items: Vec<RosterItem>) -> Vec<RosterItem> {
    let mut out: Vec<RosterItem> = Vec::new();
    for item in items {
        if matches!(item, RosterItem::Separator)
            && (out.is_empty() || matches!(out.last(), Some(RosterItem::Separator)))
        {
            continue;
        }
        out.push(item);
    }
    if matches!(out.last(), Some(RosterItem::Separator)) {
        out.pop();
    }
    out
}

fn roster_all() -> Vec<RosterMenu> {
    vec![
        RosterMenu {
            // "Awl", capitalized: the roster title feeds TWO consumers differently.
            // On native macOS, AppKit FORCIBLY substitutes the App-menu submenu's
            // displayed title with the process's own name regardless of what string
            // is set here (confirmed empirically — see the MENU-CLICK CRASH ROUND
            // notes in CLAUDE.md), so this string is INERT on native: `build_menu`
            // still passes it to `Submenu::with_items`, but nothing on screen reads
            // it. The awl-RENDERED bar (`crate::menubar` + `render/chrome/menubar.rs`,
            // shown on web/Linux where there's no OS chrome to defer to) has no such
            // constraint — it draws this string verbatim as the leftmost menu title —
            // so THIS is the one place that setting actually paints pixels.
            title: "Awl",
            items: vec![
                routed(&APP_ITEMS[0]), // About Awl
                RosterItem::Separator,
                routed(&APP_ITEMS[1]), // Settings…
                RosterItem::Separator,
                // The standard macOS App-menu Hide block (P3) — genuine OS
                // window-manager commands with no app state, the same
                // predefined class as Window's Minimize/Maximize below.
                RosterItem::Predefined(PredefinedKind::Hide),
                RosterItem::Predefined(PredefinedKind::HideOthers),
                RosterItem::Predefined(PredefinedKind::ShowAll),
                RosterItem::Separator,
                routed(&APP_ITEMS[2]), // Quit Awl
            ],
        },
        RosterMenu {
            title: "File",
            items: vec![
                routed(&FILE_ITEMS[0]), // New document
                routed(&FILE_ITEMS[1]), // Command palette
                routed(&FILE_ITEMS[2]), // Go to…
                routed(&FILE_ITEMS[3]), // Open file…
                routed(&FILE_ITEMS[4]), // Open folder…
                RosterItem::Separator,
                routed(&FILE_ITEMS[5]), // Rename file…
                routed(&FILE_ITEMS[6]), // Move file…
                routed(&FILE_ITEMS[7]), // Duplicate file
                routed(&FILE_ITEMS[8]), // Version history…
                RosterItem::Separator,
                routed(&FILE_ITEMS[9]), // Save
                routed(&FILE_ITEMS[10]), // Save and return
                RosterItem::Separator,
                routed(&FILE_ITEMS[11]), // Export as PDF
                routed(&FILE_ITEMS[12]), // Export as Word
                routed(&FILE_ITEMS[13]), // Export as HTML
            ],
        },
        RosterMenu {
            title: "Edit",
            items: vec![
                routed(&EDIT_ITEMS[0]), // Undo
                routed(&EDIT_ITEMS[1]), // Redo
                RosterItem::Separator,
                routed(&EDIT_ITEMS[2]), // Cut
                routed(&EDIT_ITEMS[3]), // Copy
                routed(&EDIT_ITEMS[4]), // Paste
                RosterItem::Separator,
                routed(&EDIT_ITEMS[5]), // Select all
                RosterItem::Separator,
                RosterItem::Submenu {
                    label: "Markdown",
                    items: MARKDOWN_ITEMS.iter().map(routed).collect(),
                },
            ],
        },
        RosterMenu {
            title: "View",
            items: vec![
                routed(&VIEW_ITEMS[0]), // Toggle page mode
                routed(&VIEW_ITEMS[1]), // Switch theme…
                RosterItem::Separator,
                routed(&VIEW_ITEMS[2]), // Zoom in
                routed(&VIEW_ITEMS[3]), // Zoom out
                routed(&VIEW_ITEMS[4]), // Reset zoom
                RosterItem::Separator,
                routed(&VIEW_ITEMS[5]), // Toggle debug
                RosterItem::Separator,
                RosterItem::Submenu {
                    label: "Page width",
                    items: VIEW_ITEMS[6..10].iter().map(routed).collect(),
                },
                RosterItem::Separator,
                routed(&VIEW_ITEMS[10]), // Toggle outline
                routed(&VIEW_ITEMS[11]), // Fold section
                routed(&VIEW_ITEMS[12]), // Collapse other sections
                routed(&VIEW_ITEMS[13]), // Toggle typewriter scroll
                routed(&VIEW_ITEMS[14]), // Toggle menu bar
            ],
        },
        RosterMenu {
            title: "Window",
            items: vec![
                RosterItem::Predefined(PredefinedKind::Minimize),
                RosterItem::Predefined(PredefinedKind::Maximize),
            ],
        },
        RosterMenu {
            title: "Help",
            items: vec![
                routed(&HELP_ITEMS[0]),
                routed(&HELP_ITEMS[1]),
                RosterItem::Separator,
                routed(&HELP_ITEMS[2]),
                routed(&HELP_ITEMS[3]),
            ],
        },
    ]
}

/// Resolve a fired muda item id (its raw [`muda::MenuId`] string) back to the
/// `Action` it routes to, via `commands::action_for_name` — the SAME catalog
/// lookup the config `[keys]` rebinder uses, so a routed item can never name
/// an action the catalog doesn't recognize. `None` for an id this table
/// doesn't own (a predefined item, or a stray/foreign event) — a silent,
/// harmless no-op at the `App::handle_menu_event` seam, never a panic.
pub fn resolve(id: &str) -> Option<Action> {
    SECTIONS
        .iter()
        .flat_map(|s| s.iter())
        .find(|r| r.id == id)
        .and_then(|r| commands::action_for_name(r.command))
}

/// The EFFECTIVE native-slot chord for a routed command NAME — CONVENTION-
/// RESOLVED, LABEL-TRUE, AND CONFIG-AWARE (`commands::menu_native_label`, e.g.
/// `"Cmd-O"` -> `"⌘O"` on Mac / `"Ctrl+O"` on Linux, `""` when the resolved
/// chord is a browser-reserved accelerator, and `""` when `keep` — the config's
/// `Config::effective_linux_keep()` — has claimed it for its emacs meaning
/// instead) for the awl-rendered menu bar's secondary column, or `""` for a
/// palette-only command with no native chord. Cross-platform (the awl bar
/// shows on web/Linux — this is the ONE label door that surface reads, so it
/// can never claim a chord the browser, or this user's `keymap` flavor / `[keys]`
/// rebind, would not actually dispatch). Reads the SAME catalog
/// [`commands::COMMANDS`] the palette does, so a menu item's chord can never
/// drift from the command it fires. `keys`/`keep` are the caller's config —
/// every real call site threads `Config::keys`/`Config::effective_linux_keep()`
/// (mirrored onto the render pipeline each `sync_view`, see
/// `render::viewstate_def::ViewState::config_keys`); a test that wants the
/// static, config-free default passes `&[]`/`&[]`.
pub fn item_chord(command: &str, keys: &[(String, Vec<String>)], keep: &[String]) -> String {
    commands::COMMANDS
        .iter()
        .find(|c| c.name == command)
        .map(|c| {
            commands::menu_native_label(
                c,
                keys,
                keep,
                crate::convention::Convention::current(),
                commands::Platform::current(),
            )
        })
        .unwrap_or_default()
}

pub fn item_chord_for_id(id: &str, keys: &[(String, Vec<String>)], keep: &[String]) -> String {
    SECTIONS
        .iter()
        .flat_map(|s| s.iter())
        .find(|r| r.id == id)
        .map(|r| item_chord(r.command, keys, keep))
        .unwrap_or_default()
}

/// The ACTUAL AppKit-displayed label for a predefined item — muda's own
/// `PredefinedMenuItemType::text()` on macOS (`&Minimize` -> "Minimize" once
/// its mnemonic `&` is stripped, `Maximize` -> "Zoom", the real macOS
/// convention muda itself special-cases per-platform). Kept as a small,
/// hand-verified pair here rather than depending on muda's private `text()`,
/// so [`print_roster`] (and therefore `scripts/smoke-menus.sh`, which drives
/// real menu clicks by exactly this displayed text) can never silently name
/// an item AppKit doesn't actually show.
pub fn predefined_label(kind: PredefinedKind) -> &'static str {
    match kind {
        PredefinedKind::Minimize => "Minimize",
        PredefinedKind::Maximize => "Zoom",
        PredefinedKind::Hide => "Hide",
        PredefinedKind::HideOthers => "Hide Others",
        PredefinedKind::ShowAll => "Show All",
    }
}

/// Print the WHOLE menu bar roster as plain, greppable lines — one per
/// CLICKABLE item (separators dropped), `<top-level menu title>\t<item
/// label>` — to stdout, then return. This is the ONE door the live-smoke
/// harness (`scripts/smoke-menus.sh`) uses to enumerate exactly what to
/// click: it shells out to `awl --print-menu-roster` and reads this output,
/// so the roster it drives can never silently drift from [`roster`] itself
/// (the same data `build_menu` translates into the real menu bar). Reachable
/// from ANY thread (pure data, like `roster` itself) — `main.rs` calls this
/// before ever touching a window, so it works even with no display attached.
#[cfg(target_os = "macos")]
pub fn print_roster() {
    for menu in roster() {
        for item in dropdown_items(&menu) {
            let label = match item {
                RosterItem::Routed { label, .. } => label,
                RosterItem::Predefined(kind) => predefined_label(kind),
                RosterItem::Submenu { label, .. } => label,
                RosterItem::Separator => continue,
            };
            println!("{}\t{}", menu.title, label);
        }
    }
}

/// The awl-rendered web/Linux bar keeps its one-level card geometry while the
/// native menu presents real nested submenus. Expanding child rows here gives
/// every Markdown verb the same route on platforms without OS menu plumbing;
/// the semantic roster above remains nested and is the source of truth.
pub fn dropdown_items(menu: &RosterMenu) -> Vec<RosterItem> {
    menu.items
        .iter()
        .flat_map(|item| match item {
            RosterItem::Submenu { items, .. } => items.clone(),
            other => vec![other.clone()],
        })
        .collect()
}

pub fn dropdown_item_enabled(item: &RosterItem, is_markdown: bool) -> bool {
    match item {
        RosterItem::Routed { id, .. } => {
            markdown_submenu_enabled(is_markdown) || !MARKDOWN_ITEMS.iter().any(|r| r.id == *id)
        }
        RosterItem::Predefined(_) | RosterItem::Submenu { .. } | RosterItem::Separator => true,
    }
}

pub fn dropdown_action(menu: &RosterMenu, index: usize, is_markdown: bool) -> Option<Action> {
    let item = dropdown_items(menu).get(index)?.clone();
    if !dropdown_item_enabled(&item, is_markdown) {
        return None;
    }
    match item {
        RosterItem::Routed { id, .. } => resolve(id),
        RosterItem::Predefined(_) | RosterItem::Submenu { .. } | RosterItem::Separator => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    use crate::menu_icons;

    fn labels(items: &[RosterItem]) -> Vec<&str> {
        items
            .iter()
            .flat_map(|item| match item {
                RosterItem::Routed { label, .. } => vec![*label],
                RosterItem::Predefined(kind) => vec![predefined_label(*kind)],
                RosterItem::Submenu { label, items } => {
                    let mut out = vec![*label];
                    out.extend(labels(items));
                    out
                }
                RosterItem::Separator => Vec::new(),
            })
            .collect()
    }

    #[test]
    fn markdown_submenu_context_changes_with_the_active_buffer_kind() {
        assert!(
            markdown_submenu_enabled(true),
            "Markdown enables its stable submenu"
        );
        assert!(
            !markdown_submenu_enabled(false),
            "plain/code buffers visibly disable the Markdown submenu"
        );
        assert!(
            markdown_submenu_enabled(true),
            "switching back re-enables it"
        );
    }

    #[test]
    fn flattened_markdown_rows_cannot_fire_in_a_non_markdown_buffer() {
        let menu = RosterMenu {
            title: "Markdown",
            items: vec![RosterItem::Routed {
                id: MARKDOWN_ITEMS[0].id,
                label: MARKDOWN_ITEMS[0].label,
                icon: false,
            }],
        };
        assert_eq!(dropdown_action(&menu, 0, false), None);
        assert_eq!(
            dropdown_action(&menu, 0, true),
            commands::action_for_name(MARKDOWN_ITEMS[0].command)
        );
    }

    #[test]
    fn markdown_context_does_not_disable_unrelated_rows() {
        let edit = roster()
            .into_iter()
            .find(|menu| menu.title == "Edit")
            .unwrap();
        let index = dropdown_items(&edit)
            .iter()
            .position(|item| matches!(item, RosterItem::Routed { .. }))
            .unwrap();
        assert!(dropdown_action(&edit, index, false).is_some());
    }

    /// LAW TEST: every routed table entry's `command` name must resolve to a
    /// real catalog `Action` — a walk of every section, so a typo'd or
    /// renamed command name in this file fails a test instead of silently
    /// building a dead menu item.
    #[test]
    fn every_routed_command_exists_in_the_catalog() {
        for section in SECTIONS {
            for r in *section {
                assert!(
                    commands::action_for_name(r.command).is_some(),
                    "menu id {:?} names {:?}, which is not a commands::COMMANDS entry",
                    r.id,
                    r.command
                );
            }
        }
    }

    #[test]
    fn every_routed_id_is_unique() {
        let mut ids: Vec<&str> = SECTIONS
            .iter()
            .flat_map(|s| s.iter())
            .map(|r| r.id)
            .collect();
        let before = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate menu id in the routed table");
    }

    /// DRIFT GUARD (single-owner): `commands::menu_section` is the CROSS-PLATFORM
    /// owner of "which menu section a command sits under" (this file is macOS-only,
    /// so the palette's File/Edit/View lenses can't reference `SECTIONS` directly —
    /// see `commands.rs`'s module note). This test pins the two representations in
    /// lockstep: every File/Edit/View menu item's command reports the MATCHING
    /// section, and the App-menu items (About/Quit) report `None`. A rename in either
    /// place fails here instead of silently splitting the menu from the palette.
    #[test]
    fn routed_sections_match_command_section() {
        for (items, expect) in [
            (APP_ITEMS, None),
            (FILE_ITEMS, Some("File")),
            (EDIT_ITEMS, Some("Edit")),
            (MARKDOWN_ITEMS, Some("Edit")),
            (HELP_ITEMS, None),
        ] {
            for r in items {
                assert_eq!(
                    commands::menu_section(r.command),
                    expect,
                    "menu item {:?} ({:?}) must agree with commands::menu_section",
                    r.id,
                    r.command
                );
            }
        }
        for r in VIEW_ITEMS {
            if r.id == "awl.page_width_settings" {
                assert_eq!(commands::menu_section(r.command), None);
            } else {
                assert_eq!(commands::menu_section(r.command), Some("View"));
            }
        }
    }

    #[test]
    fn resolve_round_trips_every_routed_entry() {
        for section in SECTIONS {
            for r in *section {
                let want = commands::action_for_name(r.command);
                assert_eq!(
                    resolve(r.id),
                    want,
                    "resolve({:?}) must match the catalog",
                    r.id
                );
            }
        }
    }

    /// LAW: the awl-RENDERED menu bar (`crate::menubar` + `render/chrome/menubar.rs`)
    /// reads THIS roster on web/Linux exactly as the macOS NSMenu bar does. This pins
    /// what the renderer needs from EVERY roster item so a future roster change can't
    /// silently leave the rendered bar with a dead row: a `Routed` item's `id` must
    /// `resolve` to a real Action (the fire path) AND its `item_chord_for_id` must not
    /// panic; a `Predefined` item must have a non-empty display label. (The renderer's
    /// own `match` over `RosterItem` — Routed / Predefined / Separator — is the
    /// no-wildcard compile-time guard; this pins the DATA each arm consumes is present.)
    #[test]
    fn renderer_consumes_every_roster_item() {
        for menu in roster() {
            for item in dropdown_items(&menu) {
                match item {
                    RosterItem::Routed { id, .. } => {
                        assert!(
                            resolve(id).is_some(),
                            "rendered bar item {id:?} resolves to no Action (dead row)"
                        );
                        // The secondary-column chord lookup must never panic (empty is
                        // fine for a palette-only command like About/Quit).
                        let _ = item_chord_for_id(id, &[], &[]);
                    }
                    RosterItem::Predefined(kind) => {
                        assert!(
                            !predefined_label(kind).is_empty(),
                            "predefined {kind:?} has no label"
                        );
                    }
                    RosterItem::Submenu { .. } | RosterItem::Separator => {}
                }
            }
        }
    }

    /// An unknown id resolves to nothing (never panics) — a predefined item's
    /// muda event (Minimize/Maximize/separator — none of which route through
    /// this table) or any stray event must be a harmless no-op.
    #[test]
    fn unknown_id_resolves_to_none() {
        assert_eq!(resolve("awl.nonexistent"), None);
        assert_eq!(resolve(""), None);
    }

    /// The ROSTER'S structure: five top-level menus, in the documented order,
    /// each carrying the exact routed/predefined/separator sequence spelled
    /// out in `roster()` above. Pure data — no muda calls, so this runs on
    /// any test thread (unlike `build_menu`, which is main-thread-only; see
    /// its own doc).
    #[test]
    fn roster_has_the_complete_top_level_menus_in_order() {
        let menus = roster();
        let titles: Vec<&str> = menus.iter().map(|m| m.title).collect();
        assert_eq!(
            titles,
            vec!["Awl", "File", "Edit", "View", "Window", "Help"]
        );
    }

    #[test]
    fn roster_app_menu_is_about_settings_hide_block_then_quit() {
        // The standard macOS App-menu shape: About · —sep— · Settings… · —sep—
        // · Hide / Hide Others / Show All (predefined) · —sep— · Quit.
        let menus = roster();
        let app = &menus[0];
        assert_eq!(
            app.items,
            vec![
                RosterItem::Routed {
                    id: "awl.about",
                    label: "About Awl",
                    icon: false
                },
                RosterItem::Separator,
                RosterItem::Routed {
                    id: "awl.settings",
                    label: "Settings…",
                    icon: false
                },
                RosterItem::Separator,
                RosterItem::Predefined(PredefinedKind::Hide),
                RosterItem::Predefined(PredefinedKind::HideOthers),
                RosterItem::Predefined(PredefinedKind::ShowAll),
                RosterItem::Separator,
                RosterItem::Routed {
                    id: "awl.quit",
                    label: "Quit Awl",
                    icon: false
                },
            ]
        );
    }

    #[test]
    fn native_file_menu_has_the_complete_writing_roster_in_order() {
        let menus = roster_for(commands::Platform::Native);
        let file = menus.iter().find(|m| m.title == "File").unwrap();
        assert_eq!(
            labels(&file.items),
            vec![
                "New document",
                "Command palette…",
                "Go to…",
                "Open file…",
                "Open folder…",
                "Rename file…",
                "Move file…",
                "Duplicate file",
                "Version history…",
                "Save",
                "Save and return",
                "Export as PDF…",
                "Export as Word…",
                "Export as HTML…",
            ]
        );
    }

    #[test]
    fn roster_window_menu_is_minimize_then_maximize_predefined_only() {
        let menus = roster();
        let window = menus.iter().find(|m| m.title == "Window").unwrap();
        assert_eq!(
            window.items,
            vec![
                RosterItem::Predefined(PredefinedKind::Minimize),
                RosterItem::Predefined(PredefinedKind::Maximize),
            ]
        );
    }

    /// Every routed table entry (APP/FILE/EDIT/VIEW) appears EXACTLY once
    /// somewhere in the roster, so `roster()` can never silently drop or
    /// duplicate a catalog-backed item relative to the routing table.
    #[test]
    fn roster_contains_every_routed_table_entry_exactly_once() {
        let menus = roster();
        let roster_ids: Vec<&str> = menus
            .iter()
            .flat_map(dropdown_items)
            .filter_map(|i| match i {
                RosterItem::Routed { id, .. } => Some(id),
                _ => None,
            })
            .collect();
        let mut table_ids: Vec<&str> = SECTIONS
            .iter()
            .flat_map(|s| s.iter())
            .map(|r| r.id)
            .collect();
        let mut sorted_roster = roster_ids.clone();
        sorted_roster.sort_unstable();
        table_ids.sort_unstable();
        assert_eq!(
            sorted_roster, table_ids,
            "roster() must place every routed table entry exactly once"
        );
    }

    /// Every routed item's LABEL matches its `commands::COMMANDS` display name
    /// exactly (menus teach the same words Cmd-P does) — EXCEPT the enumerated
    /// rows below, each a deliberate, named divergence: the macOS App-menu
    /// convention (`awl.about` / `awl.quit` append "Awl"), a shorter File-menu
    /// phrasing for the same command (`awl.switch_project` and friends), and
    /// a shorter File-menu phrasing for the same command. The three Export rows
    /// are deliberately NOT here any more: their label carried a divergence only
    /// while they completed on the spot, and now that they open a destination
    /// surface the label is the catalog name again (`menu::ellipsis_law` owns
    /// why), so the list has to shrink or the law would demand a difference that
    /// no longer exists. This is a real law for
    /// File/Edit/View (a typo there would silently diverge the menu from the
    /// palette), narrowed by name rather than left open.
    #[test]
    fn roster_routed_labels_match_the_command_catalog_display_names() {
        const EXPLICIT_MENU_LABELS: &[&str] = &[
            "awl.about",
            "awl.quit",
            "awl.switch_project",
            "awl.recent_projects",
            "awl.rename_file",
            "awl.move_file",
            "awl.duplicate_file",
            "awl.finish_buffer",
            "awl.page_width_settings",
        ];
        for menu in roster() {
            for item in dropdown_items(&menu) {
                if let RosterItem::Routed { id, label, .. } = item {
                    let r = SECTIONS
                        .iter()
                        .flat_map(|s| s.iter())
                        .find(|r| r.id == id)
                        .unwrap();
                    if EXPLICIT_MENU_LABELS.contains(&id) {
                        assert_ne!(
                            label, r.command,
                            "{id:?} is expected to differ from its bare catalog name"
                        );
                    } else {
                        assert_eq!(label, r.command);
                    }
                }
            }
        }
    }

    /// ICON FLAGS: a routed item's `icon: true` in the roster must ALWAYS
    /// resolve a real icon via `menu_icons::icon_for`, and — the converse,
    /// equally important half — an item that does NOT carry the flag must
    /// have NO icon registered for its id either. Either direction drifting
    /// (a flagged id with no drawn glyph, or a drawn glyph nobody flags) would
    /// silently diverge `roster()`'s pure data from what `build_menu` actually
    /// constructs, since `to_menu_item` only ever consults `menu_icons` when
    /// the flag is set. (macOS-only: `menu_icons` — like muda — is macOS-gated; the
    /// awl-rendered bar draws no icons, so the roster's `icon` flag is inert there.)
    #[cfg(target_os = "macos")]
    #[test]
    fn icon_flagged_routed_items_agree_with_menu_icons_exactly() {
        for menu in roster() {
            for item in menu.items {
                if let RosterItem::Routed { id, icon, .. } = item {
                    assert_eq!(
                        menu_icons::icon_for(id).is_some(),
                        icon,
                        "{id:?}: roster icon flag ({icon}) must match menu_icons::icon_for's presence"
                    );
                }
            }
        }
    }

    // ── PLATFORM-SCOPED COMMANDS: web filtering (all run on the native test binary,
    // asserting `roster_for(Platform::Web)` directly — see `commands::Platform`'s doc
    // for why a native-run test can assert the web view without an actual wasm build).

    /// `roster()` (this compiled platform, native under `cargo test`) is BYTE-IDENTICAL
    /// to `roster_for(Platform::Native)` — the compiled-platform door is exactly the
    /// explicit-platform door with `Platform::current()` filled in, never a second copy.
    #[test]
    fn roster_native_matches_roster_for_native_explicitly() {
        assert_eq!(roster(), roster_for(commands::Platform::Native));
    }

    #[test]
    fn web_roster_app_menu_keeps_about_and_settings_drops_quit_and_hide_block() {
        let menus = roster_for(commands::Platform::Web);
        let app = menus.iter().find(|m| m.title == "Awl").unwrap();
        assert_eq!(
            app.items,
            vec![
                RosterItem::Routed {
                    id: "awl.about",
                    label: "About Awl",
                    icon: false
                },
                RosterItem::Separator,
                RosterItem::Routed {
                    id: "awl.settings",
                    label: "Settings…",
                    icon: false
                },
            ]
        );
    }

    #[test]
    fn web_file_menu_prunes_history_pdf_and_save_return_without_losing_the_file_verbs() {
        let menus = roster_for(commands::Platform::Web);
        let file = menus.iter().find(|m| m.title == "File").unwrap();
        assert_eq!(
            labels(&file.items),
            vec![
                "New document",
                "Command palette…",
                "Go to…",
                "Open file…",
                "Open folder…",
                "Rename file…",
                "Move file…",
                "Duplicate file",
                "Save",
                "Export as Word…",
                "Export as HTML…",
            ]
        );
    }

    /// EXPORT ROUTING: the File-menu ids are only a third door to the existing
    /// catalog actions; they must never acquire an exporter-only dispatch path.
    #[test]
    fn file_export_ids_resolve_to_the_existing_catalog_actions() {
        assert_eq!(resolve("awl.export_pdf"), Some(Action::ExportPdf));
        assert_eq!(resolve("awl.export_word"), Some(Action::ExportWord));
        assert_eq!(resolve("awl.export_html"), Some(Action::ExportHtml));
    }

    #[test]
    fn web_roster_edit_and_view_are_untouched() {
        let native = roster_for(commands::Platform::Native);
        let web = roster_for(commands::Platform::Web);
        for title in ["Edit", "View"] {
            assert_eq!(
                native.iter().find(|m| m.title == title).unwrap().items,
                web.iter().find(|m| m.title == title).unwrap().items,
                "{title} menu must be untouched on web"
            );
        }
    }

    #[test]
    fn web_roster_drops_the_whole_window_menu() {
        let menus = roster_for(commands::Platform::Web);
        assert!(
            menus.iter().all(|m| m.title != "Window"),
            "Window must vanish on web"
        );
        let titles: Vec<&str> = menus.iter().map(|m| m.title).collect();
        assert_eq!(titles, vec!["Awl", "File", "Edit", "View", "Help"]);
    }

    #[test]
    fn web_roster_never_leaves_a_dangling_separator() {
        for menu in roster_for(commands::Platform::Web) {
            assert!(
                !matches!(menu.items.first(), Some(RosterItem::Separator)),
                "{}: leading separator",
                menu.title
            );
            assert!(
                !matches!(menu.items.last(), Some(RosterItem::Separator)),
                "{}: trailing separator",
                menu.title
            );
            assert!(
                !menu
                    .items
                    .windows(2)
                    .any(|w| matches!(w, [RosterItem::Separator, RosterItem::Separator])),
                "{}: doubled separator",
                menu.title
            );
        }
    }

    /// Every ROUTED item that survives web filtering still resolves to a real Action
    /// (the renderer-consumption law, narrowed to the filtered view) — filtering can
    /// drop a row, but never leave a dead one behind.
    #[test]
    fn web_roster_every_surviving_routed_item_resolves() {
        for menu in roster_for(commands::Platform::Web) {
            for item in dropdown_items(&menu) {
                if let RosterItem::Routed { id, .. } = item {
                    assert!(
                        resolve(id).is_some(),
                        "web roster item {id:?} resolves to no Action"
                    );
                }
            }
        }
    }
}
