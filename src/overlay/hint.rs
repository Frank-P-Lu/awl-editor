//! Shared picker footer vocabulary and per-kind control cells.

use super::OverlayKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintAction {
    pub glyph: &'static str,
    pub label: &'static str,
}

pub const HINT_SEP: &str = "   ";
pub const ARROWS_LR: &str = "\u{2190}/\u{2192}";
/// The vertical arrow pair shared by workspace lists and comparisons.
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

impl OverlayKind {
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
            OverlayKind::ExportDest => vec![
                enter("export here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            OverlayKind::ProjectBrowse => vec![
                enter("switch here"),
                key("\u{2192}", "open"),
                key("\u{2190}", "up"),
            ],
            OverlayKind::Browse => {
                vec![enter("open"), key(ARROWS_LR, "lens"), key("\u{232B}", "up")]
            }
            OverlayKind::Goto => vec![enter("open"), key(ARROWS_LR, "lens"), key("esc", "close")],
            OverlayKind::Theme => vec![enter("keep"), key("esc", "revert")],
            OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date => vec![enter("apply")],
            OverlayKind::Command => super::command_hint_actions(),
            OverlayKind::Spell => vec![enter("replace")],
            OverlayKind::Context => vec![enter("choose"), key("esc", "close")],
            OverlayKind::Keybindings => {
                vec![enter("rebind"), key("del", "reset"), key("esc", "close")]
            }
            OverlayKind::Conflict => vec![enter("read"), key("esc", "keep editing")],
            OverlayKind::History => vec![
                enter("compare"),
                key("\u{21E7}\u{21B5}", "restore"),
                key(ARROWS_LR, "lens"),
            ],
            OverlayKind::Settings => vec![enter("edit")],
            OverlayKind::Assets => vec![enter("trash"), key("esc", "close")],
            OverlayKind::Rename => vec![enter("rename"), key("esc", "cancel")],
            OverlayKind::InsertLink => vec![enter("insert link"), key("esc", "cancel")],
            OverlayKind::KeepName => vec![enter("keep"), key("esc", "cancel")],
        }
    }

    pub fn hint(self) -> String {
        format_hint(&self.hint_actions())
    }

    /// Project's flat card omits only the ascend cell; the journey-bound folder
    /// picker retains the full kind-level grammar.
    pub(crate) fn project_flat_hint(self) -> String {
        debug_assert_eq!(self, OverlayKind::Project, "no other kind is bind-scoped");
        let actions: Vec<HintAction> = self
            .hint_actions()
            .into_iter()
            .filter(|a| a.glyph != super::workspace::ERASE_GLYPH)
            .collect();
        format_hint(&actions)
    }

    pub fn range_row_actions(self) -> Vec<HintAction> {
        let mut actions = self.hint_actions();
        match actions.iter_mut().find(|a| a.glyph == ARROWS_LR) {
            Some(cell) => cell.label = RANGE_LR_LABEL,
            None => actions.push(HintAction {
                glyph: ARROWS_LR,
                label: RANGE_LR_LABEL,
            }),
        }
        actions
    }

    pub fn range_row_hint(self) -> String {
        format_hint(&self.range_row_actions())
    }
}
