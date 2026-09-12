use super::OverlayKind;
use crate::keymap::Action;

/// Decide which focused picker request an action needs before either caller
/// gathers its environment-owned payload. Live App and replay deliberately
/// provide different facts, but an action must never choose different picker
/// identities between them.
pub(crate) fn picker_kind_for(
    action: &Action,
    parked_kind: Option<OverlayKind>,
    current: Option<&super::OverlayState>,
) -> Option<OverlayKind> {
    match action {
        Action::OpenGoto
        | Action::OpenProject
        | Action::OpenRecentProjects
        | Action::OpenOutline => Some(OverlayKind::Goto),
        Action::OpenThemeMenu => Some(OverlayKind::Theme),
        Action::OpenCaretMenu => Some(OverlayKind::Caret),
        Action::OpenDictionaryMenu => Some(OverlayKind::Dictionary),
        Action::OpenKeymapMenu => Some(OverlayKind::Keymap),
        Action::OpenCommandPalette => Some(OverlayKind::Command),
        Action::OpenKeybindings => Some(OverlayKind::Keybindings),
        Action::OpenSpellSuggest => Some(OverlayKind::Spell),
        Action::OpenHistory | Action::CompareVersion => Some(OverlayKind::History),
        Action::OpenSettingsMenu => Some(OverlayKind::Settings),
        Action::OpenAssetClean => Some(OverlayKind::Assets),
        Action::OpenUserWords => Some(OverlayKind::UserWords),
        Action::OpenSearchFolder => Some(OverlayKind::SearchFolder),
        Action::OpenCredits => Some(OverlayKind::Credits),
        Action::Cancel => parked_kind,
        Action::Newline | Action::AcceptAlternate => current
            .filter(|card| card.kind == OverlayKind::Settings)
            .and_then(super::OverlayState::selected_corpus_index)
            .and_then(|index| crate::settings::visible_rows().get(index).copied())
            .and_then(|row| crate::settings::sub_overlay(row.id))
            .or(parked_kind),
        _ if current.is_some_and(|card| card.kind == OverlayKind::Settings) => {
            Some(OverlayKind::Settings)
        }
        _ => None,
    }
}
