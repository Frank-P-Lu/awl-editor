//! Settings' three focus recipients and their focus-specific teaching line.

use crate::overlay::{
    ARROWS_LR, ARROWS_UD, HINT_SEP, HintAction, OverlayKind, OverlayState, RANGE_LR_LABEL,
    format_hint,
};

/// The three keyboard recipients inside Settings. `detail_focus` remains the
/// lifecycle's coarse primary/detail projection; this names the two distinct
/// recipients within the detail pane so typing, row actions, and accessibility
/// never claim the same focus at once.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum SettingsFocus {
    #[default]
    Categories,
    Search,
    Controls,
}

impl SettingsFocus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Categories => "categories",
            Self::Search => "search",
            Self::Controls => "controls",
        }
    }

    pub(crate) fn step(self, delta: isize, has_controls: bool) -> Self {
        use SettingsFocus::{Categories, Controls, Search};
        let route: &[SettingsFocus] = if has_controls {
            &[Categories, Search, Controls]
        } else {
            &[Categories, Search]
        };
        let at = route
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0) as isize;
        route[(at + delta).rem_euclid(route.len() as isize) as usize]
    }
}

impl OverlayState {
    /// Settings' focus-specific teaching line. The journey supplies the focus
    /// recipient because that is lifecycle state; the card supplies whether a
    /// Controls stop exists and whether its selected control is a range.
    pub(crate) fn settings_focus_hint(&self, focus: SettingsFocus) -> String {
        debug_assert_eq!(self.kind, OverlayKind::Settings);
        let key = |glyph, label| HintAction { glyph, label };
        if let Some(edit) = &self.value_edit {
            return format!(
                "type {} value{}\u{21B5} apply{}esc cancel",
                edit.name.to_lowercase(),
                HINT_SEP,
                HINT_SEP
            );
        }
        let has_controls = !self.items.is_empty();
        let actions = match focus {
            SettingsFocus::Categories => vec![
                key(ARROWS_UD, "category"),
                key("tab", "search"),
                key(
                    "\u{21E7}tab",
                    if has_controls { "controls" } else { "search" },
                ),
                key("esc", "close"),
            ],
            SettingsFocus::Search => vec![
                key("type", "filter"),
                key(
                    "tab",
                    if has_controls {
                        "controls"
                    } else {
                        "categories"
                    },
                ),
                key("\u{21E7}tab", "categories"),
                key("esc", "close"),
            ],
            SettingsFocus::Controls => {
                let mut actions = vec![key(ARROWS_UD, "control")];
                if self.selected_range().is_some() {
                    actions.push(key(ARROWS_LR, RANGE_LR_LABEL));
                }
                actions.extend([
                    key("\u{21B5}", "edit"),
                    key("tab", "categories"),
                    key("\u{21E7}tab", "search"),
                    key("esc", "close"),
                ]);
                actions
            }
        };
        format_hint(&actions)
    }
}
