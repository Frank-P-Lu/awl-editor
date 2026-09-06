use super::kind::OverlayKind;

impl OverlayKind {
    /// A SMALL, CARET-ANCHORED INSERTION CARD rather than a takeover of the
    /// room. This is deliberately independent of [`Self::keeps_backdrop_crisp`]:
    /// the table-dimensions card does not preview the live document, it merely
    /// declines to make a modest insertion choice recede the whole canvas.
    ///
    /// Exhaustive so a new overlay cannot silently inherit the exemption.
    pub fn is_local_insertion_card(self) -> bool {
        match self {
            OverlayKind::TableDims => true,
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest
            | OverlayKind::SearchFolder => false,
        }
    }
}
