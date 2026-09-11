//! The shared contract for every place awl asks the writer to choose a location.
//!
//! A location picker is deliberately not a generic filesystem dialog.  This
//! module owns the parts the consumers genuinely share — directory scope, folder
//! rows, query/focus, breadcrumb presentation, and back navigation — while the
//! typed operation still owns its final effect and filesystem side effect.

use super::OverlayKind;

/// The action that becomes possible only after a location is explicitly
/// accepted.  It is vocabulary for the route roster, not an effect: effects
/// retain their operation-specific payloads and interpreters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitVerb {
    OpenFile,
    MoveHere,
    SaveCopyHere,
    ExportHere,
    SwitchHere,
}

/// Every consumer of a user-chosen location, including the macOS-native doors.
/// The native rows remain native surfaces; naming them here makes their final
/// verbs auditable beside the in-app navigators rather than leaving a second,
/// unrostered family in `app/menu.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub(crate) enum LocationConsumer {
    Browse,
    MoveDestination,
    SaveCopyDestination,
    ExportDestination,
    ProjectBrowse,
    NativeOpen,
    NativeSaveCopy,
    NativeExportDocx,
    NativeExportHtml,
    NativeExportPdf,
}

/// Whether a navigator's path is relative to the active writing folder or an
/// absolute path under the configured workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocationScope {
    ActiveRoot,
    Workspace,
}

impl LocationConsumer {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 10] = [
        Self::Browse,
        Self::MoveDestination,
        Self::SaveCopyDestination,
        Self::ExportDestination,
        Self::ProjectBrowse,
        Self::NativeOpen,
        Self::NativeSaveCopy,
        Self::NativeExportDocx,
        Self::NativeExportHtml,
        Self::NativeExportPdf,
    ];

    /// The in-app card this consumer uses, if it has one.  Save Copy reuses the
    /// export destination card but remains a separate consumer because its
    /// filename/overwrite constraints and final writer are different.
    pub(crate) fn overlay_kind(self) -> Option<OverlayKind> {
        match self {
            Self::Browse => Some(OverlayKind::Browse),
            Self::MoveDestination => Some(OverlayKind::MoveDest),
            Self::SaveCopyDestination | Self::ExportDestination => Some(OverlayKind::ExportDest),
            Self::ProjectBrowse => Some(OverlayKind::ProjectBrowse),
            Self::NativeOpen
            | Self::NativeSaveCopy
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => None,
        }
    }

    pub(crate) fn commit_verb(self) -> CommitVerb {
        match self {
            Self::Browse | Self::NativeOpen => CommitVerb::OpenFile,
            Self::MoveDestination => CommitVerb::MoveHere,
            Self::SaveCopyDestination | Self::NativeSaveCopy => CommitVerb::SaveCopyHere,
            Self::ExportDestination
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => CommitVerb::ExportHere,
            Self::ProjectBrowse => CommitVerb::SwitchHere,
        }
    }

    pub(crate) fn scope(self) -> LocationScope {
        match self {
            Self::ProjectBrowse => LocationScope::Workspace,
            Self::Browse
            | Self::MoveDestination
            | Self::SaveCopyDestination
            | Self::ExportDestination
            | Self::NativeOpen
            | Self::NativeSaveCopy
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => LocationScope::ActiveRoot,
        }
    }

    /// Folder rows are a navigation step, never an operation commit.  Browse
    /// is the one member that additionally renders files, which only open on
    /// its explicit accept.
    pub(crate) fn folders_only(self) -> bool {
        !matches!(self, Self::Browse | Self::NativeOpen)
    }

    pub(crate) fn shows_relative_breadcrumb(self) -> bool {
        matches!(
            self,
            Self::MoveDestination | Self::SaveCopyDestination | Self::ExportDestination
        )
    }

    pub(crate) fn supports_back(self) -> bool {
        self.overlay_kind().is_some()
    }

    pub(crate) fn allows_typed_new_folder(self) -> bool {
        matches!(self, Self::ExportDestination)
    }

    pub(crate) fn commit_label(self) -> &'static str {
        match self.commit_verb() {
            CommitVerb::OpenFile => "open",
            CommitVerb::MoveHere => "move here",
            CommitVerb::SaveCopyHere => "save a copy here",
            CommitVerb::ExportHere => "export here",
            CommitVerb::SwitchHere => "switch here",
        }
    }

    #[cfg(test)]
    pub(crate) fn native_menu_id(self) -> Option<&'static str> {
        match self {
            Self::NativeOpen => Some("awl.open"),
            Self::NativeSaveCopy => Some("awl.save_copy"),
            Self::NativeExportPdf => Some("awl.export_pdf"),
            Self::NativeExportDocx => Some("awl.export_word"),
            Self::NativeExportHtml => Some("awl.export_html"),
            _ => None,
        }
    }
}

/// The navigator family represented by `kind`.  `ExportDest` has two typed
/// operations, so callers resolving its commitment use [`consumer_for_card`]
/// to preserve that distinction.
pub(crate) fn navigator_for(kind: OverlayKind) -> Option<LocationConsumer> {
    match kind {
        OverlayKind::Browse => Some(LocationConsumer::Browse),
        OverlayKind::MoveDest => Some(LocationConsumer::MoveDestination),
        OverlayKind::ExportDest => Some(LocationConsumer::ExportDestination),
        OverlayKind::ProjectBrowse => Some(LocationConsumer::ProjectBrowse),
        _ => None,
    }
}

/// The exact operation represented by an existing card.  The `save_copy` bit
/// is carried through every relevel by `Journey`, so this is safe to ask only
/// at explicit acceptance time.
pub(crate) fn consumer_for_card(card: &super::OverlayState) -> Option<LocationConsumer> {
    match card.kind {
        OverlayKind::ExportDest if card.save_copy => Some(LocationConsumer::SaveCopyDestination),
        kind => navigator_for(kind),
    }
}

/// The macOS File-menu panel route, resolved from its catalog action.  This is
/// intentionally pure so the roster law can sweep it on Linux and wasm too.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn native_consumer_for_action(
    action: &crate::keymap::Action,
) -> Option<LocationConsumer> {
    use crate::keymap::Action;
    match action {
        Action::OpenBrowse => Some(LocationConsumer::NativeOpen),
        Action::SaveCopy => Some(LocationConsumer::NativeSaveCopy),
        Action::ExportPdf => Some(LocationConsumer::NativeExportPdf),
        Action::ExportWord => Some(LocationConsumer::NativeExportDocx),
        Action::ExportHtml => Some(LocationConsumer::NativeExportHtml),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Roster-derived consumer × verb law.  The no-wildcard matches make a new
    /// route or a changed native panel fail to compile here instead of silently
    /// joining the product without an explicit commitment verb.
    #[test]
    fn every_location_consumer_declares_its_commit_verb_and_navigation_contract() {
        for consumer in LocationConsumer::ALL {
            match consumer {
                LocationConsumer::Browse
                | LocationConsumer::MoveDestination
                | LocationConsumer::SaveCopyDestination
                | LocationConsumer::ExportDestination
                | LocationConsumer::ProjectBrowse
                | LocationConsumer::NativeOpen
                | LocationConsumer::NativeSaveCopy
                | LocationConsumer::NativeExportDocx
                | LocationConsumer::NativeExportHtml
                | LocationConsumer::NativeExportPdf => {}
            }
            match consumer.commit_verb() {
                CommitVerb::OpenFile
                | CommitVerb::MoveHere
                | CommitVerb::SaveCopyHere
                | CommitVerb::ExportHere
                | CommitVerb::SwitchHere => {}
            }
            if let Some(kind) = consumer.overlay_kind() {
                assert!(
                    consumer.supports_back(),
                    "{consumer:?}: in-app navigation needs Back"
                );
                assert_eq!(
                    navigator_for(kind),
                    Some(match consumer {
                        LocationConsumer::SaveCopyDestination => {
                            LocationConsumer::ExportDestination
                        }
                        other => other,
                    }),
                    "{consumer:?}: card routing must stay in the shared owner"
                );
            }
        }
        assert!(LocationConsumer::ExportDestination.allows_typed_new_folder());
        assert!(!LocationConsumer::ProjectBrowse.allows_typed_new_folder());
        assert!(LocationConsumer::MoveDestination.shows_relative_breadcrumb());
        assert!(!LocationConsumer::ProjectBrowse.shows_relative_breadcrumb());
    }

    #[test]
    fn every_native_location_panel_has_the_rostered_action_and_verb() {
        for consumer in LocationConsumer::ALL {
            let Some(id) = consumer.native_menu_id() else {
                continue;
            };
            let action = crate::menu::resolve(id)
                .unwrap_or_else(|| panic!("{consumer:?}: {id:?} does not resolve"));
            assert!(
                crate::menu::opens_native_panel(id),
                "{consumer:?}: {id:?} must stay a native location panel"
            );
            assert_eq!(
                native_consumer_for_action(&action),
                Some(consumer),
                "{consumer:?}: native action must route through the shared roster"
            );
            assert!(
                matches!(
                    consumer.commit_verb(),
                    CommitVerb::OpenFile | CommitVerb::SaveCopyHere | CommitVerb::ExportHere
                ),
                "{consumer:?}: a native file panel must promise a file operation"
            );
        }
    }
}
