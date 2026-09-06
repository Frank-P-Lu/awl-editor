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
    /// Does this kind draw NO teaching footer at all — the pocket palette's own
    /// world-list grammar, whose right-click idiom is ambient and needs no
    /// lesson? Filtering and Enter/Esc keep working silently either way; this
    /// only gates the DISPLAYED line, never the capability. One predicate
    /// shared by [`Self::hint_actions`] and [`Self::range_row_actions`] so the
    /// two cannot drift into a state where one still authors a partial line
    /// (an arrows-only cell with no `type to filter` lead) the other has
    /// already dropped in full.
    fn draws_no_teaching_footer(self) -> bool {
        matches!(self, OverlayKind::Context)
    }

    pub fn hint_actions(self) -> Vec<HintAction> {
        if self.draws_no_teaching_footer() {
            return Vec::new();
        }
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
            | OverlayKind::Date
            | OverlayKind::Keymap => vec![enter("apply")],
            OverlayKind::Command => super::command_hint_actions(),
            OverlayKind::Spell => vec![enter("replace")],
            // Never actually formatted into a line: `hint_actions` returns
            // empty for Context before this arm is reached. Kept truthful
            // anyway — Enter still chooses a row and Esc still closes, this
            // is the real capability roster, only the teaching text is gone.
            OverlayKind::Context => vec![enter("choose"), key("esc", "close")],
            OverlayKind::Keybindings => {
                vec![enter("rebind"), key("del", "reset"), key("esc", "close")]
            }
            OverlayKind::Conflict => vec![enter("read"), key("esc", "keep editing")],
            // Unreachable in practice: `Credits` is a workspace kind
            // (`workspace_shape().is_some()`), so `foot_hint_scoped` always
            // routes its footer through `rail_hint_actions`/
            // `detail_hint_actions` instead of this flat-picker line. Kept
            // truthful anyway (the universal ↵-follows-the-filter-lead shape
            // every kind's flat hint obeys), mirroring Conflict's own —
            // the same "read-only, nothing to commit" content beside it.
            OverlayKind::Credits => vec![enter("read"), key("esc", "close")],
            OverlayKind::History => vec![
                enter("compare"),
                key("\u{21E7}\u{21B5}", "restore"),
                key(ARROWS_LR, "lens"),
            ],
            OverlayKind::Settings => vec![enter("edit")],
            OverlayKind::Assets => vec![enter("trash"), key("esc", "close")],
            OverlayKind::UserWords => vec![enter("forget"), key("esc", "close")],
            OverlayKind::Rename => vec![enter("rename"), key("esc", "cancel")],
            OverlayKind::InsertLink => vec![enter("insert link"), key("esc", "cancel")],
            OverlayKind::KeepName => vec![enter("keep"), key("esc", "cancel")],
            // Unreachable in practice: `foot_hint_scoped` returns
            // `TableDimsEdit::prompt`'s own live readout before this arm is
            // reached. Kept truthful anyway.
            OverlayKind::TableDims => vec![enter("insert"), key("esc", "cancel")],
            OverlayKind::SearchFolder => {
                vec![enter("open"), key("esc", "close")]
            }
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
        if self.draws_no_teaching_footer() {
            // No context row ever carries a range cell (`OverlayState::
            // selected_range` reads `rows[].range`, which the context-menu
            // constructor never sets), so this variant is unreachable in
            // product data. Answering it with the same empty line
            // `hint_actions` does — rather than the bare `←/→ adjust` cell
            // the fallthrough below would push — keeps "no footer for a
            // pocket palette" one fact instead of two.
            return Vec::new();
        }
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
