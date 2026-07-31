enum_with_all! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OverlayKind {
        Goto,
        Project,
        Browse,
        Theme,
        Caret,
        Dictionary,
        CjkLang,
        Date,
        MoveDest,
        Command,
        Spell,
        Keybindings,
        History,
        Settings,
        Assets,
        Rename,
        InsertLink,
        KeepName,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptDisposition {
    Navigate,
    ValuePick,
    StayOpen,
}

impl OverlayKind {
    pub fn from_mode(mode: &str) -> Option<OverlayKind> {
        Self::ALL.iter().copied().find(|k| k.as_str() == mode)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            OverlayKind::Goto => "goto",
            OverlayKind::Project => "switch",
            OverlayKind::Browse => "browse",
            OverlayKind::Theme => "theme",
            OverlayKind::Caret => "caret",
            OverlayKind::Dictionary => "dictionary",
            OverlayKind::CjkLang => "cjk_lang",
            OverlayKind::Date => "date",
            OverlayKind::MoveDest => "move",
            OverlayKind::Command => "command",
            OverlayKind::Spell => "spell",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "history",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "assets",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert_link",
            OverlayKind::KeepName => "keep_version",
        }
    }

    pub fn accept_disposition(self) -> AcceptDisposition {
        use AcceptDisposition::*;
        match self {
            OverlayKind::Goto
            | OverlayKind::Browse
            | OverlayKind::Project
            | OverlayKind::MoveDest
            | OverlayKind::Spell
            | OverlayKind::History
            | OverlayKind::Command => Navigate,
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date => ValuePick,
            OverlayKind::Assets | OverlayKind::Keybindings | OverlayKind::Settings => StayOpen,
            OverlayKind::Rename => Navigate,
            OverlayKind::InsertLink => Navigate,
            OverlayKind::KeepName => Navigate,
        }
    }

    #[allow(dead_code)] // consumed only by overlay::tests's runtime roster sweep today.
    pub fn row_meta_roster(self) -> &'static [super::RowMetaTag] {
        use super::RowMetaTag::*;
        match self {
            OverlayKind::Goto => &[GotoFile, GotoHeading],
            OverlayKind::Command => &[Plain, CommandHidden, CommandSetting],
            OverlayKind::Spell => &[Plain, SpellAdd],
            OverlayKind::History => &[History],
            OverlayKind::Project
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::Keybindings
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName => &[Plain],
        }
    }

    pub fn hides_dotfiles(self) -> bool {
        matches!(
            self,
            OverlayKind::Goto | OverlayKind::Browse | OverlayKind::MoveDest | OverlayKind::Project
        )
    }

    pub const MAX_SUGGESTIONS: usize = 5;

    pub fn window_rows(self) -> usize {
        match self {
            OverlayKind::Spell => Self::MAX_SUGGESTIONS + 1,
            OverlayKind::Theme => crate::theme::THEMES.len(),
            // ITEM 114 — a SUMMONED WORKSPACE is bounded by the canvas, not by a
            // card-sized row count: it already occupies the viewport, so a
            // twelve-row cap would leave two thirds of it empty and scroll a list
            // that fits. The canvas bound is item 181's `fit_item_rows`, applied
            // in the renderer where the canvas is actually known; naming the whole
            // corpus here is what lets that bound BE the binding one — the same
            // arrangement the theme picker's own roster already uses above.
            OverlayKind::Settings => crate::settings::SETTINGS.len(),
            _ => 12,
        }
    }

    pub fn hint_actions(self) -> Vec<HintAction> {
        let mut actions = vec![HintAction {
            glyph: "type",
            label: "to filter",
        }];
        actions.extend(self.kind_actions());
        actions
    }

    fn kind_actions(self) -> Vec<HintAction> {
        let enter = |label| HintAction {
            glyph: "\u{21B5}",
            label,
        };
        let key = |glyph, label| HintAction { glyph, label };
        match self {
            OverlayKind::Project => vec![
                enter("select"),
                key(ARROWS_LR, "lens"),
                key("\u{232B}", "up"),
            ],
            OverlayKind::MoveDest => vec![
                enter("move here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            OverlayKind::Browse => {
                vec![enter("open"), key(ARROWS_LR, "lens"), key("\u{232B}", "up")]
            }
            OverlayKind::Goto => vec![enter("open"), key(ARROWS_LR, "lens")],
            OverlayKind::Theme => vec![enter("keep"), key("esc", "revert")],
            OverlayKind::Caret => vec![enter("apply")],
            OverlayKind::Dictionary => vec![enter("apply")],
            OverlayKind::CjkLang => vec![enter("apply")],
            OverlayKind::Date => vec![enter("apply")],
            OverlayKind::Command => vec![enter("run"), key(ARROWS_LR, "lens")],
            OverlayKind::Spell => vec![enter("replace")],
            OverlayKind::Keybindings => {
                vec![enter("rebind"), key("del", "reset"), key("esc", "close")]
            }
            OverlayKind::History => {
                vec![enter("restore"), key("tab", "diff"), key(ARROWS_LR, "lens")]
            }
            // ITEM 114 — the Settings ROWS pane is the workspace's DETAIL stage,
            // so `esc` is a BACK to the category rail (the table's
            // `WorkspaceDetail × Cancel → Primary` cell), and `←/→` steps the
            // rail's own category rather than an anonymous "lens". The footer is
            // awl's only statement of what a key does (ACCESSIBILITY.md), so
            // these two cells change with the presentation, not after it.
            OverlayKind::Settings => {
                vec![
                    enter("edit"),
                    key(ARROWS_LR, "category"),
                    key("esc", "back"),
                ]
            }
            OverlayKind::Assets => vec![enter("trash"), key("esc", "close")],
            OverlayKind::Rename => vec![enter("rename"), key("esc", "cancel")],
            OverlayKind::InsertLink => vec![enter("insert link"), key("esc", "cancel")],
            OverlayKind::KeepName => vec![enter("keep"), key("esc", "cancel")],
        }
    }

    pub fn hint(self) -> String {
        format_hint(&self.hint_actions())
    }

    /// The foot hint while a summoned WORKSPACE's PRIMARY list — its navigation
    /// rail — holds focus (item 114). The rows pane's own hint is
    /// [`Self::hint_actions`]; this is the other stage's, and the two differ in
    /// exactly the keys that differ: on the rail `↑/↓` steps categories and `esc`
    /// leaves for the editor, while on the rows `↑/↓` steps rows and `esc` comes
    /// back here.
    ///
    /// Wildcard-free, like every other per-kind statement here: a kind that is
    /// not drawn as a workspace still has to say what its rail would advertise,
    /// which is nothing, and it can never be reached because
    /// [`crate::overlay::OverlayState::foot_hint`] gates on
    /// [`Self::workspace_shell`].
    pub fn rail_hint_actions(self) -> Vec<HintAction> {
        let enter = |label| HintAction {
            glyph: "\u{21B5}",
            label,
        };
        let key = |glyph, label| HintAction { glyph, label };
        match self {
            OverlayKind::Settings => vec![
                key(ARROWS_UD, "category"),
                enter("settings"),
                key("esc", "close"),
            ],
            OverlayKind::History
            | OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName => Vec::new(),
        }
    }

    pub fn range_row_hint(self) -> String {
        let mut actions = self.hint_actions();
        match actions.iter_mut().find(|a| a.glyph == ARROWS_LR) {
            Some(cell) => cell.label = RANGE_LR_LABEL,
            None => actions.push(HintAction {
                glyph: ARROWS_LR,
                label: RANGE_LR_LABEL,
            }),
        }
        format_hint(&actions)
    }

    pub fn empty_corpus_message(self) -> &'static str {
        match self {
            OverlayKind::History => "no history yet",
            OverlayKind::Spell => "no suggestions",
            OverlayKind::Browse => "this folder is empty",
            OverlayKind::Goto | OverlayKind::Project | OverlayKind::MoveDest => "no files here",
            OverlayKind::Assets => "no unused assets",
            OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Command
            | OverlayKind::Keybindings
            | OverlayKind::Settings => "no matches",
            OverlayKind::Rename => "no matches",
            OverlayKind::InsertLink => "no matches",
            OverlayKind::KeepName => "no matches",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            OverlayKind::Goto => "go to",
            OverlayKind::Project => "switch project",
            OverlayKind::Browse => "browse",
            OverlayKind::Theme => "themes",
            OverlayKind::Caret => "caret style",
            OverlayKind::MoveDest => "move note",
            OverlayKind::Dictionary => "dictionary",
            OverlayKind::CjkLang => "ambiguous cjk",
            OverlayKind::Date => "date format",
            OverlayKind::Command => "commands",
            OverlayKind::Spell => "spelling",
            OverlayKind::Keybindings => "keybindings",
            OverlayKind::History => "version history",
            OverlayKind::Settings => "settings",
            OverlayKind::Assets => "unused assets",
            OverlayKind::Rename => "rename",
            OverlayKind::InsertLink => "insert link",
            OverlayKind::KeepName => "keep version",
        }
    }

    pub fn row_path_splits(self) -> bool {
        match self {
            OverlayKind::InsertLink => true,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::MoveDest
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::KeepName => false,
        }
    }

    pub fn draws_title_prefix(self) -> bool {
        !matches!(
            self,
            OverlayKind::Rename | OverlayKind::InsertLink | OverlayKind::KeepName
        )
    }

    pub const SETTINGS_MARKER_PREFIX: &'static str = "§ ";

    pub const HEADING_MARKER_PREFIX: &'static str = "❡ ";

    pub fn empty_lens_message(self, lens: &str) -> Option<&'static str> {
        match (self, lens) {
            (OverlayKind::Goto, "recent") => Some("no recent files yet"),
            (OverlayKind::Goto, "headings") => Some("no headings yet"),
            (OverlayKind::Project, "recent") => Some("no recent projects yet"),
            (_, "all") => None,
            _ => Some("nothing here"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintAction {
    pub glyph: &'static str,
    pub label: &'static str,
}

pub const HINT_SEP: &str = "   ";

pub const ARROWS_LR: &str = "\u{2190}/\u{2192}";

pub const ARROWS_UD: &str = "\u{2191}/\u{2193}";

pub const RANGE_LR_LABEL: &str = "adjust";

pub const PIN_TAG: &str = "pinned";

pub fn format_hint(actions: &[HintAction]) -> String {
    actions
        .iter()
        .map(|a| format!("{} {}", a.glyph, a.label))
        .collect::<Vec<_>>()
        .join(HINT_SEP)
}
