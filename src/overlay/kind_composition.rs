use super::kind::OverlayKind;

impl OverlayKind {
    /// A brief contextual choice that keeps the surrounding writing readable.
    ///
    /// Commands answers "what should happen here?" and Insert-link asks for one
    /// destination while the sentence being edited remains useful context. Neither
    /// relocates attention into a sustained workspace, so neither earns the
    /// full-canvas frost used by a takeover picker. The renderer still applies its
    /// composition-owned local treatment: a Pane card fully backs its contents
    /// and needs nothing behind it; Bars, Ruled, and Diagonal leave gaps where
    /// document ink could interleave, so they receive the shared footprint frost.
    ///
    /// This is deliberately independent of `keeps_backdrop_crisp`: these cards do
    /// not preview the document, they retain it as context. Exhaustive so a new
    /// surface cannot silently inherit the exception.
    pub fn retains_readable_room(self) -> bool {
        match self {
            OverlayKind::Command | OverlayKind::InsertLink => true,
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
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Credits
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::UserWords
            | OverlayKind::Rename
            | OverlayKind::KeepName
            | OverlayKind::Context
            | OverlayKind::ExportDest
            | OverlayKind::TableDims
            | OverlayKind::SearchFolder => false,
        }
    }

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
